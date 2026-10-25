use super::*;

fn setup() -> (Harness, String, String) {
    let mut h = Harness::new();
    let folder = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/counter.sdPlugin");
    let mut plugin = crate::plugins::parse(&folder, &std::fs::read(folder.join("manifest.json")).unwrap()).unwrap();
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
    let (mut h,plugin,action) = setup();
    let size = vec2(1366.0,768.0);
    for (i,value) in [(0,11),(1,22)] {
        let output = h.warm(size);
        let label = format!("Compteur DeckForge › Compteur");
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
            if let Some(out) = std::env::var_os("NOVYDECK_UI_OUTPUT") {h.png(size,output,&PathBuf::from(out).join("plugin-blocks-fr.png"));}
        }
        assert!(positions.len() >= 2,"block labels are not visible");
        h.click(size,positions[i]);
        let context = h.app.block_inspector.as_ref().expect("direct click opens session").context.clone();
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
