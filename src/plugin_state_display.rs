//! Ephemeral, opt-in plugin state observations. No state is inferred from binding defaults.
use std::collections::HashMap;
use serde_json::Value;
use crate::model::{Config, Key, PluginStateTarget, Step};

#[derive(Clone, Debug, PartialEq)]
pub struct Source {
    pub context: String,
    pub plugin: String,
    pub action: String,
    pub settings: Value,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Observation { Unknown, Confirmed(bool), Local(bool), SourceReported(bool) }
impl Observation {
    pub fn active(self) -> Option<bool> {
        match self { Self::Unknown => None, Self::Confirmed(v) | Self::Local(v) | Self::SourceReported(v) => Some(v) }
    }
}

/// Paths use the same indices and branch slots as plugin_runtime::virtual_context.
/// Disabled actions and long-press actions have no virtual runtime binding.
pub fn block_sources(key: &Key) -> Vec<(Vec<usize>, String, String)> {
    fn walk(steps: &[Step], parent: &[usize], out: &mut Vec<(Vec<usize>, String, String)>) {
        for (i, step) in steps.iter().enumerate() {
            if step.is_disabled() { continue; }
            let mut path = parent.to_vec(); path.push(i);
            match step {
                Step::PluginAction { plugin, action, .. } => out.push((path, plugin.clone(), action.clone())),
                Step::Repeat { steps, .. } => { path.push(0); walk(steps, &path, out); }
                Step::Toggle { a, b } => { let mut branch = path.clone(); branch.push(0); walk(a, &branch, out); path.push(1); walk(b, &path, out); }
                Step::IfCondition { then_steps, else_steps, .. } | Step::IfWindow { then_steps, else_steps, .. } | Step::IfVariable { then_steps, else_steps, .. } => {
                    let mut branch = path.clone(); branch.push(0); walk(then_steps, &branch, out); path.push(1); walk(else_steps, &path, out);
                }
                _ => {}
            }
        }
    }
    let mut out = Vec::new(); walk(&key.steps, &[], &mut out); out
}
fn block_at<'a>(steps: &'a [Step], path: &[usize]) -> Option<&'a Step> {
    if path.is_empty() || path.len() > 32 { return None; }
    let step = steps.get(path[0])?;
    if step.is_disabled() { return None; }
    if path.len() == 1 { return matches!(step, Step::PluginAction { .. }).then_some(step); }
    let branch = *path.get(1)?;
    let nested = match step {
        Step::Repeat { steps, .. } if branch == 0 => steps,
        Step::Toggle { a, .. } if branch == 0 => a,
        Step::Toggle { b, .. } if branch == 1 => b,
        Step::IfCondition { then_steps, .. } | Step::IfWindow { then_steps, .. } | Step::IfVariable { then_steps, .. } if branch == 0 => then_steps,
        Step::IfCondition { else_steps, .. } | Step::IfWindow { else_steps, .. } | Step::IfVariable { else_steps, .. } if branch == 1 => else_steps,
        _ => return None,
    };
    block_at(nested, &path[2..])
}
pub fn target_valid(key: &Key, target: &PluginStateTarget) -> bool {
    match target {
        PluginStateTarget::Direct => key.plugin.as_ref().is_some_and(|binding| !binding.plugin.is_empty() && !binding.action.is_empty() && binding.settings.is_object()),
        PluginStateTarget::Block { path } => block_at(&key.steps, path).is_some(),
    }
}
/// None denotes a removed/invalid source, never a false (OFF) observation.
pub fn source(config: &Config, index: usize, target: &PluginStateTarget) -> Option<Source> {
    let key = config.keys().get(index)?;
    if !target_valid(key, target) { return None; }
    match target {
        PluginStateTarget::Direct => {
            let binding = key.plugin.as_ref()?;
            Some(Source { context: format!("deckforge-{}-{}-{}", config.active, config.page, index), plugin: binding.plugin.clone(), action: binding.action.clone(), settings: binding.settings.clone() })
        }
        PluginStateTarget::Block { path } => {
            let Step::PluginAction { plugin, action, settings, .. } = block_at(&key.steps, path)? else { return None };
            let profile = config.profiles.get(config.active)?;
            let identity = crate::engine::identity(&profile.name, &config.page, index);
            Some(Source { context: crate::plugin_runtime::virtual_context(&identity, path), plugin: plugin.clone(), action: action.clone(), settings: settings.clone() })
        }
    }
}

#[derive(Default)]
pub struct Cache { values: HashMap<String, (Source, Observation)> }
impl Cache {
    /// Observe only a source selected on the currently visible configuration.
    /// A stale/foreign context is never interpreted as a new instance.
    pub fn observe(&mut self, config: &Config, owner: &str, message: &Value) -> Option<(usize, Observation)> {
        let context = message["context"].as_str()?;
        let (index, current) = config.keys().iter().enumerate().find_map(|(index, key)| {
            let target = key.plugin_state.as_ref()?;
            let current = source(config, index, target)?;
            (current.context == context && current.plugin == owner).then_some((index, current))
        })?;
        Some((index, self.record(&current, owner, message)))
    }
    /// A short press supplies a visual fallback only while no real state is reported.
    /// The source identity prevents a fallback from leaking across settings or pages.
    pub fn toggle_local(&mut self, config: &Config, index: usize, key: &Key) -> Option<bool> {
        let source = source(config, index, key.plugin_state.as_ref()?)?;
        let active = match self.lookup_source(&source) {
            Observation::Unknown => true,
            Observation::Local(previous) => !previous,
            Observation::Confirmed(_) | Observation::SourceReported(_) => return None,
        };
        self.values.insert(source.context.clone(), (source, Observation::Local(active)));
        Some(active)
    }
    /// Call with the current, trusted source (not fields copied from a plugin message).
    /// Unexpected owner, action or context cannot overwrite a previous observation.
    pub fn record(&mut self, source: &Source, owner: &str, message: &Value) -> Observation {
        if owner != source.plugin || message["context"].as_str() != Some(&source.context)
            || message.get("action").is_some_and(|a| a.as_str() != Some(&source.action)) {
            return self.lookup_source(source);
        }
        let observation = match message["event"].as_str() {
            // A plugin may repeatedly report unavailable state (e.g. RTSS without its DLL).
            // Such a report must not erase the short-press local visual fallback.
            Some("novydeckStateUnknown") => match self.lookup_source(source) {
                Observation::Local(active) => Observation::Local(active),
                _ => Observation::Unknown,
            },
            Some("setState") => {
                let value = match message["payload"]["state"].as_u64() { Some(0) => false, Some(1) => true, _ => { self.values.remove(&source.context); return Observation::Unknown; } };
                match message["payload"]["novydeckStateSource"].as_str() {
                    Some("confirmed") => Observation::Confirmed(value),
                    Some("local") => Observation::Local(value),
                    None if message["payload"].get("novydeckStateSource").is_none() => Observation::SourceReported(value),
                    _ => {self.values.remove(&source.context);return Observation::Unknown;},
                }
            }
            _ => return self.lookup_source(source),
        };
        if observation == Observation::Unknown { self.values.remove(&source.context); }
        else { self.values.insert(source.context.clone(), (source.clone(), observation)); }
        observation
    }
    /// Only current configured instances are eligible; stale settings reset to Unknown.
    pub fn lookup_source(&self, source: &Source) -> Observation {
        self.values.get(&source.context).filter(|(saved, _)| saved == source).map_or(Observation::Unknown, |(_, state)| *state)
    }
    pub fn lookup(&self, config: &Config, index: usize, key: &Key) -> Observation {
        key.plugin_state.as_ref().and_then(|target| source(config, index, target)).map_or(Observation::Unknown, |source| self.lookup_source(&source))
    }
    pub fn apply(&self, config: &Config, index: usize, key: &Key) -> Key {
        match key.state_display.as_ref() {
            Some(display) => display.apply(key, self.lookup(config, index, key).active().unwrap_or(false)),
            None => key.clone(),
        }
    }
    pub fn clear_plugin(&mut self, plugin: &str) { self.values.retain(|_, (source, _)| source.plugin != plugin); }
    pub fn retain(&mut self, config: &Config) {
        self.values.retain(|_, (saved, _)| config.keys().iter().enumerate().any(|(i, key)|
            key.plugin_state.as_ref().and_then(|target| source(config, i, target)).is_some_and(|current| current == *saved)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{PluginBinding, StateDisplay, StateFace};
    use serde_json::json;
    fn fixture() -> (Config, Key) {
        let mut config = Config::default();
        let key = Key { image: "custom.png".into(), plugin: Some(PluginBinding { plugin: "owner".into(), action: "switch".into(), settings: json!({"id":1}), state: 0 }),
            plugin_state: Some(PluginStateTarget::Direct), state_display: Some(StateDisplay { on: StateFace { image: Some("on.png".into()), ..Default::default() }, off: StateFace { image: Some("builtin:speakers".into()), ..Default::default() }, ..Default::default() }), ..Key::default() };
        config.keys_mut()[0] = key.clone(); (config, key)
    }
    #[test] fn serde_opt_in_and_faces_reload() {
        let (config, key) = fixture();
        let mut legacy = serde_json::to_value(&key).unwrap(); legacy.as_object_mut().unwrap().remove("plugin_state");
        let legacy_bytes = serde_json::to_string(&Key::default()).unwrap();
        assert!(!legacy_bytes.contains("plugin_state"));
        assert_eq!(serde_json::to_string(&serde_json::from_str::<Key>(&legacy_bytes).unwrap()).unwrap(), legacy_bytes);
        assert!(serde_json::from_value::<Key>(legacy.clone()).unwrap().plugin_state.is_none());
        assert_eq!(serde_json::to_value(serde_json::from_value::<Key>(legacy).unwrap()).unwrap().get("plugin_state"), None);
        let loaded: Config = serde_json::from_value(serde_json::to_value(config).unwrap()).unwrap();
        assert_eq!(loaded.keys()[0].plugin_state, Some(PluginStateTarget::Direct));
        assert_eq!(loaded.keys()[0].state_display, key.state_display);
    }
    #[test] fn provenance_unknown_and_safe_faces() {
        let (config, key) = fixture(); let s = source(&config, 0, key.plugin_state.as_ref().unwrap()).unwrap(); let mut cache = Cache::default();
        assert_eq!(cache.apply(&config, 0, &key).display_kind, crate::audio::builtin_kind("builtin:speakers"));
        assert!(cache.apply(&config, 0, &key).runtime_badge.is_none());
        let msg = |state: Value, origin: Option<&str>| { let mut v = json!({"event":"setState","context":s.context,"action":s.action,"payload":{"state":state}}); if let Some(origin) = origin { v["payload"]["novydeckStateSource"] = json!(origin); } v };
        assert_eq!(cache.record(&s, "owner", &msg(json!(1), Some("confirmed"))), Observation::Confirmed(true));
        assert_eq!(cache.observe(&config, "other", &msg(json!(0), None)), None);
        assert_eq!(cache.observe(&config, "owner", &msg(json!(1), Some("confirmed"))), Some((0, Observation::Confirmed(true))));
        assert_eq!(cache.apply(&config, 0, &key).image, "on.png");
        assert_eq!(cache.record(&s, "owner", &msg(json!(0), Some("local"))), Observation::Local(false));
        let off = cache.apply(&config, 0, &key);
        assert_eq!(off.display_kind, crate::audio::builtin_kind("builtin:speakers")); assert!(off.image.is_empty());
        for bad in [msg(json!(2), None), msg(json!("1"), None), msg(json!(true), None)] {
            assert_eq!(cache.record(&s, "owner", &msg(json!(0), Some("local"))), Observation::Local(false));
            assert_eq!(cache.record(&s, "owner", &bad), Observation::Unknown);
        }
        assert_eq!(cache.record(&s, "owner", &msg(json!(0), Some("local"))), Observation::Local(false));
        assert_eq!(cache.record(&s, "owner", &msg(json!(1), Some("forged"))), Observation::Unknown);
        cache.record(&s, "owner", &msg(json!(0), Some("local")));
        assert_eq!(cache.record(&s, "other", &msg(json!(1), None)), Observation::Local(false));
        let mut alien = msg(json!(1), None); alien["action"] = json!("alien"); assert_eq!(cache.record(&s, "owner", &alien), Observation::Local(false));
        alien["action"] = json!("switch"); alien["context"] = json!("other"); assert_eq!(cache.record(&s, "owner", &alien), Observation::Local(false));
        assert_eq!(cache.record(&s, "owner", &msg(json!(1), None)), Observation::SourceReported(true));
        assert_eq!(cache.record(&s, "owner", &json!({"event":"novydeckStateUnknown","context":s.context,"action":s.action})), Observation::Unknown);
        assert_eq!(cache.toggle_local(&config, 0, &key), Some(true));
        assert_eq!(cache.record(&s, "owner", &json!({"event":"novydeckStateUnknown","context":s.context,"action":s.action})), Observation::Local(true));
        assert_eq!(cache.record(&s, "owner", &msg(json!(0), Some("confirmed"))), Observation::Confirmed(false));
        assert_eq!(cache.record(&s, "owner", &json!({"event":"novydeckStateUnknown","context":s.context,"action":s.action})), Observation::Unknown);
        assert_eq!(cache.apply(&config, 0, &key).display_kind, crate::audio::builtin_kind("builtin:speakers"));
    }
    #[test] fn paths_settings_and_instances() {
        let (mut config, key) = fixture();
        let mk = |id: u8| Step::PluginAction { plugin: "owner".into(), action: "switch".into(), settings: json!({"id":id}), label: String::new() };
        config.keys_mut()[0].steps = vec![mk(1), Step::Repeat { count: 1, steps: vec![mk(2)] }];
        config.keys_mut()[0].plugin_state = Some(PluginStateTarget::Block { path: vec![0] });
        let mut second = key.clone(); second.steps = config.keys()[0].steps.clone(); second.plugin_state = Some(PluginStateTarget::Block { path: vec![1,0,0] }); config.keys_mut()[1] = second;
        let a = source(&config, 0, config.keys()[0].plugin_state.as_ref().unwrap()).unwrap();
        let b = source(&config, 1, config.keys()[1].plugin_state.as_ref().unwrap()).unwrap();
        assert_ne!(a.context, b.context); assert_ne!(a.settings, b.settings);
        let mut cache = Cache::default();
        cache.record(&a, "owner", &json!({"event":"setState","context":a.context,"payload":{"state":1}}));
        cache.record(&b, "owner", &json!({"event":"setState","context":b.context,"payload":{"state":0}}));
        assert_eq!(cache.lookup(&config, 0, config.keys().first().unwrap()), Observation::SourceReported(true));
        assert_eq!(cache.lookup(&config, 1, &config.keys()[1]), Observation::SourceReported(false));
        config.keys_mut()[0].steps[0] = mk(3);
        assert_eq!(cache.lookup(&config, 0, &config.keys()[0]), Observation::Unknown);
        assert_eq!(cache.lookup(&config, 1, &config.keys()[1]), Observation::SourceReported(false));
        cache.clear_plugin("owner"); assert_eq!(cache.lookup(&config, 1, &config.keys()[1]), Observation::Unknown);
        assert!(!target_valid(&key, &PluginStateTarget::Block { path: vec![] }));
        assert!(!target_valid(&key, &PluginStateTarget::Block { path: vec![999] }));
    }
}
