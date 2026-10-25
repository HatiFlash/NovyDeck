use super::*;

/// A session is tied to the draft tree, never to an applied key or a runtime macro context.
pub(super) struct BlockInspector {
    pub context: String,
    pub profile: usize,
    pub page: String,
    pub key: usize,
    pub path: Vec<usize>,
    pub plugin: String,
    pub action: String,
    pub tree: serde_json::Value,
}

fn structure(steps: &[Step]) -> serde_json::Value {
    fn strip(value: &mut serde_json::Value) {
        match value {
            serde_json::Value::Object(fields) => {
                if fields.get("type").and_then(|v| v.as_str()) == Some("plugin_action") { fields.remove("settings"); }
                for item in fields.values_mut() { strip(item); }
            }
            serde_json::Value::Array(items) => for item in items { strip(item); },
            _ => (),
        }
    }
    let mut value = serde_json::to_value(steps).unwrap_or(serde_json::Value::Null);
    strip(&mut value);
    value
}

impl App {
    pub(super) fn block_inspector_close(&mut self) {
        if let Some(old) = self.block_inspector.take() {
            if let Some(manager) = &self.plugins { let _ = manager.close_virtual_inspector(&old.context); }
        }
    }
    pub(super) fn block_inspector_valid(&self) -> bool {
        let Some(s) = self.block_inspector.as_ref() else { return false };
        if self.config.active != s.profile || self.config.page != s.page || self.selected != s.key || !self.show_workflow { return false; }
        let Some(key) = self.config.keys().get(s.key) else { return false };
        let Some(Step::PluginAction {plugin,action,..}) = extras::step_mut(&mut key.steps.clone(), &s.path).cloned() else { return false };
        plugin == s.plugin && action == s.action && structure(&key.steps) == s.tree
    }
    pub(super) fn block_inspector_open(&mut self, path: Vec<usize>) {
        self.block_inspector_close();
        let Some(key) = self.config.keys().get(self.selected) else { return };
        let Some(Step::PluginAction {plugin,action,settings,..}) = extras::step_mut(&mut key.steps.clone(), &path).cloned() else { return };
        let Some(manager) = self.plugins.as_ref() else { return };
        if !manager.catalog.iter().any(|p|p.uuid == plugin && p.prerequisite.is_none()
            && p.actions.iter().any(|a|a.uuid == action && a.inspector.as_ref().or(p.inspector.as_ref()).is_some())) { return; }
        self.block_inspector_nonce = self.block_inspector_nonce.wrapping_add(1);
        let context = format!("novydeck-inspector-block-{}-{}", std::process::id(), self.block_inspector_nonce);
        let session = BlockInspector { context: context.clone(), profile: self.config.active,
            page: self.config.page.clone(), key: self.selected, path,
            plugin: plugin.clone(), action: action.clone(), tree: structure(&key.steps) };
        let visible = crate::plugin_runtime::Visible { context, plugin, action,
            index: 0, settings, state: 0 };
        match manager.open_virtual_inspector(visible) {
            Ok(()) => self.block_inspector = Some(session),
            Err(error) => self.fail(crate::i18n::trf("Inspecteur indisponible : {error}", &[("error", &error)])),
        }
    }
    /// Returns true for every reserved block-inspector context, including stale ones.
    pub(super) fn block_inspector_message(&mut self, plugin: &str, message: &serde_json::Value) -> bool {
        let Some(context) = message["context"].as_str() else { return false };
        if !context.starts_with("novydeck-inspector-block-") { return false; }
        if !self.block_inspector_valid() { self.block_inspector_close(); return true; }
        let Some(s) = self.block_inspector.as_ref() else { return true };
        if context != s.context || plugin != s.plugin || message["action"].as_str().is_some_and(|a| a != s.action) { return true; }
        let name = message["event"].as_str().unwrap_or("");
        if name == "getSettings" {
            let Some(Step::PluginAction {settings,..}) = extras::step_mut(&mut self.config.keys()[s.key].steps.clone(), &s.path).cloned() else { return true };
            let v = crate::plugin_runtime::Visible { context: context.into(), plugin: s.plugin.clone(),
                action: s.action.clone(), index: 0, settings: settings.clone(), state: 0 };
            if let Some(manager) = &self.plugins { manager.send(plugin, app_settings_event(&v,&crate::plugin_runtime::Device::virtual_device(),&settings,None)); }
        } else if name == "setSettings" {
            let Some(payload) = message["payload"].as_object() else { return true };
            let settings = serde_json::Value::Object(payload.clone());
            let s = self.block_inspector.as_ref().unwrap();
            let (key,path) = (s.key,s.path.clone());
            let before = self.config.clone();
            if let Some(Step::PluginAction {settings: target,..}) = extras::step_mut(&mut self.config.keys_mut()[key].steps, &path) {
                *target = settings.clone();
                if let Err(error) = store::save(&self.path, &self.config) {
                    self.config = before;
                    self.fail(crate::i18n::trf("Réglages du plugin non sauvegardés : {error}", &[("error", &error)]));
                } else {
                    if let Some(manager) = &self.plugins { manager.virtual_settings(context, settings); }
                    self.edited();
                }
            }
        }
        true
    }
}
