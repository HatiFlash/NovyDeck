use super::*;

fn setup() -> (Harness, String, String) {
    let mut h = Harness::new();
    let folder = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/counter.sdPlugin");
    let mut plugin = crate::plugins::parse(&folder, &std::fs::read(folder.join("manifest.json")).unwrap()).unwrap();
    plugin.name = "Atelier Témoin".into();
    plugin.inspector = Some("inspector.html".into()); // Offline routing fixture; no WebView is asserted.
    let (id, action) = (plugin.uuid.clone(), plugin.actions[0].uuid.clone());
    h.app.plugins = Some(crate::plugin_runtime::Manager::new(vec![plugin]));
    h.app.show_workflow = true;
    h.app.config.keys_mut()[0].steps = (0..2).map(|n| Step::PluginAction {
        plugin: id.clone(), action: action.clone(), settings: serde_json::json!({}), label: format!("Bloc {n}"),
    }).collect();
    (h,id,action)
}

#[test]
fn two_draft_blocks_click_properties_route_without_action_and_persist() {
    for lang in ["fr","en"] {
    let (mut h,plugin,action) = setup();
    crate::i18n::set_language(lang);h.app.preferences.language=lang.into();
    let size = vec2(1366.0,768.0);
    for (i,value) in [(0,11),(1,22)] {
        let output = h.warm(size);
        let label = "Atelier Témoin › Compteur";
        let property_left = egui::containers::panel::PanelState::load(&h.ctx,egui::Id::new("workshop_properties_panel"))
            .map_or(size.x,|panel|panel.rect.left());
        let mut positions: Vec<_> = output.shapes.iter().filter_map(|shape| {
            if let egui::Shape::Text(t) = &shape.shape {
                if t.galley.job.text.contains(&label) && t.pos.x < property_left && shape.clip_rect.contains(t.pos) {
                    return Some(t.pos + vec2(10.0,8.0));
                }
            }
            None
        }).collect();
        positions.sort_by(|a,b|a.y.total_cmp(&b.y));
        if i == 0 {
            if let Some(out) = std::env::var_os("NOVYDECK_UI_OUTPUT") {h.png(size,output,&PathBuf::from(out).join(format!("plugin-blocks-{lang}.png")));}
        }
        assert!(positions.len() >= 2,"block labels are not visible");
        h.click(size,positions[i]);
        let output=h.warm(size);
        let button=text_position(&output.shapes,crate::i18n::tr("Réglages du plugin")).expect("explicit plugin settings button must be visible");
        if i==0 {
            if let Some(out)=std::env::var_os("NOVYDECK_UI_OUTPUT") {h.png(size,output,&PathBuf::from(out).join(format!("plugin-block-properties-{lang}.png")));}
            h.app.block_inspector_close();h.click(size,button);h.warm(size);
        }
        let context = h.app.block_inspector.as_ref().expect("direct click or properties button opens session").context.clone();
        assert_eq!(h.app.block_inspector.as_ref().unwrap().path,vec![i]);
        assert!(context.starts_with("novydeck-inspector-block-"));
        assert!(!context.starts_with("novydeck-block-"));
        h.app.plugin_message(&plugin,&serde_json::json!({"event":"setSettings","context":context,"payload":{"count":value}}));
        assert!(matches!(&h.app.config.keys()[0].steps[i],Step::PluginAction {settings,..} if settings["count"]==value));
    }
    let saved = crate::store::load(&h.dir.path().join("config.json")).unwrap();
    for (i,value) in [(0,11),(1,22)] {
        assert!(matches!(&saved.keys()[0].steps[i],Step::PluginAction {settings,..} if settings["count"]==value));
    }
    assert_eq!(action,"org.deckforge.counter.increment");
    }
}

#[test]
fn same_action_different_settings_reorder_and_profile_replacement_invalidate_session() {
    let (mut h,plugin,_) = setup();
    for (index,step) in h.app.config.keys_mut()[0].steps.iter_mut().enumerate() {
        if let Step::PluginAction {settings,label,..}=step {*settings=serde_json::json!({"id":index});*label="Identical label".into();}
    }
    h.app.block_inspector_open(vec![0]);let old=h.app.block_inspector.as_ref().unwrap().context.clone();
    h.app.config.keys_mut()[0].steps.swap(0,1);let before=h.app.config.keys()[0].steps.clone();
    h.app.plugin_message(&plugin,&serde_json::json!({"event":"setSettings","context":old,"payload":{"id":99}}));
    assert!(h.app.block_inspector.is_none());assert_eq!(h.app.config.keys()[0].steps,before);
    h.app.block_inspector_open(vec![0]);let old=h.app.block_inspector.as_ref().unwrap().context.clone();
    h.app.config.profiles[0].name="Replacement profile".into();
    h.app.plugin_message(&plugin,&serde_json::json!({"event":"setSettings","context":old,"payload":{"id":99}}));
    assert!(h.app.block_inspector.is_none());assert_eq!(h.app.config.keys()[0].steps,before);
}

#[test]
fn own_settings_refresh_signature_but_malformed_action_cannot_write() {
    let (mut h,plugin,action)=setup();let applied=h.app.applied.clone();
    h.app.block_inspector_open(vec![0]);let context=h.app.block_inspector.as_ref().unwrap().context.clone();
    h.app.plugin_message(&plugin,&serde_json::json!({"event":"setSettings","context":context,"payload":{"value":1}}));
    assert!(h.app.block_inspector_valid());assert!(h.app.gate.dirty);assert_eq!(serde_json::to_value(&h.app.applied).unwrap(),serde_json::to_value(&applied).unwrap());
    let before=h.app.config.keys()[0].steps.clone();
    h.app.plugin_message(&plugin,&serde_json::json!({"event":"setSettings","context":context,"action":42,"payload":{"value":99}}));
    assert_eq!(h.app.config.keys()[0].steps,before);
    h.app.plugin_message(&plugin,&serde_json::json!({"event":"setSettings","context":context,"action":action,"payload":{"value":2}}));
    assert!(h.app.block_inspector_valid());
    let loaded=crate::store::load(&h.app.path).unwrap();
    assert!(matches!(&loaded.keys()[0].steps[0],Step::PluginAction{settings,..} if settings["value"]==2));
}

#[test]
fn valid_block_inspectors_preserve_global_settings_protocol() {
    let (mut h,plugin,_)=setup();h.app.block_inspector_open(vec![0]);
    let context=h.app.block_inspector.as_ref().unwrap().context.clone();
    h.app.plugin_message(&plugin,&serde_json::json!({"event":"setGlobalSettings","context":context,"payload":{"test_global":17}}));
    let data:serde_json::Value=serde_json::from_slice(&std::fs::read(h.app.path.with_file_name("plugin-settings.json")).unwrap()).unwrap();
    assert_eq!(data[&plugin]["test_global"],17);
    assert!(!h.app.block_inspector_message(&plugin,&serde_json::json!({"event":"getGlobalSettings","context":context})),"valid global requests must reach the existing SDK handler");
}

#[test]
fn stale_sessions_never_write_new_block_after_reorder_or_remove() {
    let (mut h,plugin,_) = setup();
    h.app.block_inspector_open(vec![0]);
    let old = h.app.block_inspector.as_ref().unwrap().context.clone();
    h.app.config.keys_mut()[0].steps.swap(0,1);
    h.app.plugin_message(&plugin,&serde_json::json!({"event":"setSettings","context":old,"payload":{"count":99}}));
    assert!(h.app.block_inspector.is_none());
    assert!(h.app.config.keys()[0].steps.iter().all(|s|matches!(s,Step::PluginAction {settings,..} if settings==&serde_json::json!({}))));
    h.app.block_inspector_open(vec![0]);
    let old = h.app.block_inspector.as_ref().unwrap().context.clone();
    h.app.config.keys_mut()[0].steps.remove(0);
    h.app.plugin_message(&plugin,&serde_json::json!({"event":"setSettings","context":old,"payload":{"count":99}}));
    assert!(h.app.block_inspector.is_none());
    h.app.block_inspector_open(vec![0]);
    let old = h.app.block_inspector.as_ref().unwrap().context.clone();
    h.app.selected = 1;
    h.app.plugin_message(&plugin,&serde_json::json!({"event":"setSettings","context":old,"payload":{"count":99}}));
    assert!(h.app.block_inspector.is_none());
}

#[test]
fn googlehome_control_settings_persist_across_reload_and_keep_neighbour_targets() {
    let mut h = Harness::new();
    let folder = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/counter.sdPlugin");
    let mut plugin = crate::plugins::parse(&folder, &std::fs::read(folder.join("manifest.json")).unwrap()).unwrap();
    plugin.uuid = "com.novydeck.googlehome".into();
    plugin.name = "Google Home témoin".into();
    plugin.inspector = Some("inspector.html".into());
    plugin.actions[0].uuid = "com.novydeck.googlehome.control".into();
    let plugin_id = plugin.uuid.clone();
    h.app.plugins = Some(crate::plugin_runtime::Manager::new(vec![plugin]));
    h.app.show_workflow = true;
    let neighbour_settings = serde_json::json!({"scope":"group","targets":["id-neighbour"],"mode":"on"});
    h.app.config.keys_mut()[0].steps = vec![
        Step::PluginAction {
            plugin: plugin_id.clone(), action: "com.novydeck.googlehome.control".into(),
            settings: serde_json::json!({}), label: "Bloc cible".into(),
        },
        Step::PluginAction {
            plugin: plugin_id.clone(), action: "com.novydeck.googlehome.control".into(),
            settings: neighbour_settings.clone(), label: "Bloc voisin".into(),
        },
    ];
    // Opening the properties must not silently convert an existing brightness
    // action or an unconfigured action to toggle.
    if let Step::PluginAction { settings, .. } = &mut h.app.config.keys_mut()[0].steps[0] {
        *settings = serde_json::json!({"mode":"brightness","brightness":42,"scope":"device","target":"id1"});
    }
    let size = vec2(1366.0, 768.0);
    crate::i18n::set_language("fr");
    h.app.preferences.language = "fr".into();
    h.warm(size); // initialise le contexte de l'atelier
    let output = h.warm(size);
    let card = output.shapes.iter().find_map(|shape| {
        if let egui::Shape::Text(text) = &shape.shape {
            if text.galley.job.text.contains("Google Home témoin") && text.pos.x < 1000.0
                && shape.clip_rect.contains(text.pos) { return Some(text.pos + vec2(10.0, 8.0)); }
        }
        None
    }).expect("la carte Google Home est visible");
    h.click(size, card);
    let output = h.warm(size);
    assert!(text_position(&output.shapes, crate::i18n::tr("Luminosité (réglages du plugin)")).is_some(),
        "le panneau de propriétés doit être ouvert sur le bloc choisi");
    if let Some(dir) = std::env::var_os("NOVYDECK_UI_OUTPUT") {
        h.png(size, output, &PathBuf::from(dir).join("googlehome-block-properties-fr.png"));
    }
    assert!(matches!(&h.app.config.keys()[0].steps[0], Step::PluginAction {settings,..}
        if settings["mode"] == "brightness" && settings["brightness"] == 42));
    if let Step::PluginAction { settings, .. } = &mut h.app.config.keys_mut()[0].steps[0] {
        *settings = serde_json::json!({});
    }
    h.warm(size);
    assert!(matches!(&h.app.config.keys()[0].steps[0], Step::PluginAction {settings,..} if settings == &serde_json::json!({})));
    h.app.block_inspector_open(vec![0]);
    let context = h.app.block_inspector.as_ref().expect("googlehome block inspector opens").context.clone();
    assert!(context.starts_with("novydeck-inspector-block-"));

    // Real setSettings sent by the inspector: scope/targets/mode together.
    let payload = serde_json::json!({"scope":"group","targets":["id1","id2"],"mode":"off"});
    h.app.plugin_message(&plugin_id, &serde_json::json!({"event":"setSettings","context":context,"payload":payload}));
    assert!(matches!(&h.app.config.keys()[0].steps[0],
        Step::PluginAction {settings,..} if settings == &payload));
    assert!(matches!(&h.app.config.keys()[0].steps[1],
        Step::PluginAction {settings,..} if settings == &neighbour_settings));
    let output = h.warm(size);
    let group_label = crate::i18n::trf("Groupe : {count} appareils", &[("count", &2)]);
    assert!(text_position(&output.shapes, &group_label).is_some(),
        "le bloc configuré doit indiquer le nombre d'appareils choisis");
    if let Some(dir) = std::env::var_os("NOVYDECK_UI_OUTPUT") {
        h.png(size, output, &PathBuf::from(dir).join("googlehome-block-group-off-fr.png"));
    }

    // Persisted on disk and still intact after reload.
    let saved = crate::store::load(&h.dir.path().join("config.json")).unwrap();
    assert!(matches!(&saved.keys()[0].steps[0],
        Step::PluginAction {settings,..} if settings == &payload));
    assert!(matches!(&saved.keys()[0].steps[1],
        Step::PluginAction {settings,..} if settings == &neighbour_settings));

    // Mode change via the same setSettings path keeps targets/scope untouched.
    // Reopen the block inspector after the UI preview, as a user would.
    h.app.block_inspector_open(vec![0]);
    let session = h.app.block_inspector.as_ref().expect("inspector reopens after settings are saved").context.clone();
    let switched = serde_json::json!({"scope":"group","targets":["id1","id2"],"mode":"toggle"});
    h.app.plugin_message(&plugin_id, &serde_json::json!({"event":"setSettings","context":session,"payload":switched}));
    let saved = crate::store::load(&h.dir.path().join("config.json")).unwrap();
    assert!(matches!(&saved.keys()[0].steps[0],
        Step::PluginAction {settings,..} if settings["mode"] == "toggle" && settings["targets"] == serde_json::json!(["id1","id2"]) && settings["scope"] == "group"));
    assert!(matches!(&saved.keys()[0].steps[1],
        Step::PluginAction {settings,..} if settings == &neighbour_settings));
}

#[test]
fn googlehome_block_properties_english_capture() {
    let Some(dir) = std::env::var_os("NOVYDECK_UI_OUTPUT") else { return; };
    let mut h = Harness::new();
    let folder = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/counter.sdPlugin");
    let mut plugin = crate::plugins::parse(&folder, &std::fs::read(folder.join("manifest.json")).unwrap()).unwrap();
    plugin.uuid = "com.novydeck.googlehome".into();
    plugin.name = "Google Home demo".into();
    plugin.actions[0].uuid = "com.novydeck.googlehome.control".into();
    h.app.plugins = Some(crate::plugin_runtime::Manager::new(vec![plugin]));
    h.app.show_workflow = true;
    h.app.config.keys_mut()[0].steps = vec![Step::PluginAction {
        plugin: "com.novydeck.googlehome".into(),
        action: "com.novydeck.googlehome.control".into(),
        settings: serde_json::json!({"mode":"off","scope":"group","targets":["demo-1","demo-2"]}),
        label: "Demo group (offline)".into(),
    }];
    crate::i18n::set_language("en");
    h.app.preferences.language = "en".into();
    let size = vec2(1366.0, 768.0);
    h.warm(size);
    h.ctx.data_mut(|d| d.insert_temp(egui::Id::new("workshop_path"), vec![0_usize]));
    let output = h.warm(size);
    assert!(text_position(&output.shapes, crate::i18n::tr("État à l’appui")).is_some());
    assert!(text_position(&output.shapes, crate::i18n::trf("Groupe : {count} appareils", &[("count", &2)])).is_some());
    h.png(size, output, &PathBuf::from(dir).join("googlehome-block-properties-en.png"));
}
