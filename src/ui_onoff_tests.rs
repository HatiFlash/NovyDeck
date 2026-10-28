use super::*;
use egui::{vec2, Event as Input};

fn second_add(shapes: &[egui::epaint::ClippedShape]) -> egui::Pos2 {
    let title = text_position(shapes, "2e appui").expect("carte du deuxième appui");
    fn find(shape: &egui::Shape, title: egui::Pos2) -> Option<egui::Pos2> {
        match shape {
            egui::Shape::Text(t) if t.galley.job.text == "Ajouter" && (t.pos.x - title.x).abs() < 180.0 => Some(t.pos + t.galley.size() * 0.5),
            egui::Shape::Vec(v) => v.iter().find_map(|s| find(s, title)),
            _ => None,
        }
    }
    shapes.iter().find_map(|s| find(&s.shape, title).filter(|p| s.clip_rect.contains(*p))).expect("bouton Ajouter du deuxième appui")
}

#[test]
fn onoff_f9_conversion_and_second_action_edit_in_main_inspector() {
    let mut h = Harness::new();
    let size = vec2(1366.0 / 1.25, 768.0 / 1.25);
    h.ctx.set_pixels_per_point(1.25);
    h.app.config.keys_mut()[0].label = "F9".into();
    h.app.config.keys_mut()[0].steps = vec![Step::Hotkey { keys: "F9".into() }];
    let output = h.warm(size);
    let pos = text_position(&output.shapes, "Actions ON/OFF").expect("ON/OFF dans Action, sans Atelier");
    h.click(size, pos);
    let output = h.warm(size);
    assert!(matches!(&h.app.config.keys()[0].steps[..], [Step::Toggle { a, b }] if a == &vec![Step::Hotkey { keys: "F9".into() }] && b.is_empty()));
    assert!(text_position(&output.shapes, "1er appui").is_some());
    if let Some(dir) = std::env::var_os("NOVYDECK_UI_OUTPUT") { h.png(size, output.clone(), &std::path::PathBuf::from(dir).join("onoff-compact-cards.png")); }
    let action2 = text_position(&output.shapes, "2e appui").expect("carte 2e appui");
    h.click(size, action2);
    let output = h.warm(size);
    assert_eq!(h.app.config.keys()[0].label, "F9");
    assert!(h.app.config.keys()[0].state_display.is_none());
    h.click(size, second_add(&output.shapes));
    let output = h.warm(size);
    let pos = text_position(&output.shapes, "Raccourci clavier").expect("raccourci clavier au menu Ajouter");
    h.click(size, pos);
    let output = h.warm(size);
    assert!(matches!(&h.app.config.keys()[0].steps[..], [Step::Toggle { b, .. }] if matches!(&b[..], [Step::Hotkey { .. }])), "steps after catalog click: {:?}, menu at {:?}", h.app.config.keys()[0].steps, pos);
    let branch = text_position(&output.shapes, "2e appui").expect("onglet Action 2 toujours visible");
    let mut output = output;
    for _ in 0..12 {
        if text_position(&output.shapes, "Enregistrer…").is_some() { break; }
        output = h.frame(size, vec![
            Input::PointerMoved(branch + egui::vec2(85.0, 40.0)),
            Input::MouseWheel { unit: egui::MouseWheelUnit::Point, delta: vec2(0.0, -55.0), modifiers: egui::Modifiers::NONE },
        ]);
    }
    if let Some(dir) = std::env::var_os("NOVYDECK_UI_OUTPUT") {
        h.png(size, output.clone(), &std::path::PathBuf::from(dir).join("onoff-compact-action2.png"));
    }
    let record = text_position(&output.shapes, "Enregistrer…").expect("champ Action 2 accessible par défilement");
    h.click(size, record + egui::vec2(-90.0, 0.0));
    h.frame(size, vec![Input::Key { key: egui::Key::A, physical_key: Some(egui::Key::A), pressed: true, repeat: false, modifiers: egui::Modifiers { ctrl: true, command: true, ..Default::default() } }]);
    h.frame(size, vec![Input::Text("F10".into())]);
    h.warm(size);
    assert!(matches!(&h.app.config.keys()[0].steps[..], [Step::Toggle { a, b }] if matches!(&a[..], [Step::Hotkey { keys }] if keys == "F9") && matches!(&b[..], [Step::Hotkey { keys }] if keys == "F10")), "edited steps: {:?}", h.app.config.keys()[0].steps);
    assert!(!h.app.show_workflow);
}

#[test]
fn onoff_conversion_preserves_plugin_state_block_target() {
    let mut h = Harness::new();
    let size = vec2(1366.0, 768.0);
    let key = &mut h.app.config.keys_mut()[0];
    key.steps = vec![Step::PluginAction {
        plugin: "test.plugin".into(), action: "test.action".into(),
        settings: serde_json::json!({}), label: String::new(),
    }];
    key.plugin_state = Some(crate::model::PluginStateTarget::Block { path: vec![0] });
    let output = h.warm(size);
    let enable = text_position(&output.shapes, "Actions ON/OFF").expect("main inspector ON/OFF button");
    h.click(size, enable);
    let key = &h.app.config.keys()[0];
    assert!(matches!(&key.steps[..], [Step::Toggle { a, b }] if a.len() == 1 && b.is_empty()));
    assert_eq!(key.plugin_state, Some(crate::model::PluginStateTarget::Block { path: vec![0, 0, 0] }));
}

#[test]
fn onoff_navigation_remains_a_root_action() {
    let mut h = Harness::new();
    let size = vec2(1366.0, 768.0);
    h.app.config.keys_mut()[0].steps = vec![Step::PageNext];
    let output = h.warm(size);
    let enable = text_position(&output.shapes, "Actions ON/OFF").expect("disabled ON/OFF button explains the restriction");
    h.click(size, enable);
    assert!(matches!(h.app.config.keys()[0].steps.as_slice(), [Step::PageNext]));
}

#[test]
fn onoff_branch_removal_rebases_and_clears_plugin_state_target() {
    let mut h = Harness::new();
    let size = vec2(1600.0, 1000.0);
    let plugin = |action: &str| Step::PluginAction {
        plugin: "test.plugin".into(), action: action.into(),
        settings: serde_json::json!({}), label: action.into(),
    };
    let key = &mut h.app.config.keys_mut()[0];
    key.steps = vec![Step::Toggle { a: vec![plugin("first"), plugin("second")], b: vec![] }];
    key.plugin_state = Some(crate::model::PluginStateTarget::Block { path: vec![0, 0, 1] });
    let output = h.warm(size);
    let remove = text_position(&output.shapes, "Retirer").expect("first branch action removable in the main inspector");
    h.click(size, remove);
    let key = &h.app.config.keys()[0];
    assert!(matches!(&key.steps[..], [Step::Toggle { a, .. }] if matches!(&a[..], [Step::PluginAction { action, .. }] if action == "second")));
    assert_eq!(key.plugin_state, Some(crate::model::PluginStateTarget::Block { path: vec![0, 0, 0] }));
    let output = h.warm(size);
    let remove = text_position(&output.shapes, "Retirer").expect("remaining plugin action removable");
    h.click(size, remove);
    let key = &h.app.config.keys()[0];
    assert!(matches!(&key.steps[..], [Step::Toggle { a, .. }] if a.is_empty()));
    assert!(key.plugin_state.is_none(), "removed plugin must not observe another source");
}

#[test]
fn onoff_second_action_can_be_chosen_from_main_catalog_menu() {
    let mut h = Harness::new();
    let size = vec2(1366.0, 768.0);
    h.app.config.keys_mut()[0].steps = vec![Step::Toggle {
        a: vec![Step::Hotkey { keys: "F9".into() }], b: vec![],
    }];
    let output = h.warm(size);
    h.click(size, text_position(&output.shapes, "2e appui").expect("second action tab"));
    let output = h.warm(size);
    h.click(size, second_add(&output.shapes));
    let output = h.warm(size);
    h.click(size, text_position(&output.shapes, "Attendre").expect("delay action in picker"));
    assert!(matches!(&h.app.config.keys()[0].steps[..], [Step::Toggle { a, b }]
        if matches!(&a[..], [Step::Hotkey { keys }] if keys == "F9")
        && matches!(&b[..], [Step::Delay { .. }])));
    assert!(!h.app.show_workflow);
}

#[test]
fn onoff_existing_branches_and_extra_steps_are_not_silently_lost() {
    let mut h = Harness::new();
    let size = vec2(1366.0, 768.0);
    let a = vec![Step::Hotkey { keys: "F9".into() }, Step::Delay { ms: 125 }];
    let b = vec![Step::Hotkey { keys: "F10".into() }, Step::Delay { ms: 250 }];
    h.app.config.keys_mut()[0].steps = vec![Step::Toggle { a: a.clone(), b: b.clone() }];
    let output = h.warm(size);
    assert!(text_position(&output.shapes, "1er appui").is_some());
    assert!(text_position(&output.shapes, "2e appui").is_some());
    assert!(matches!(&h.app.config.keys()[0].steps[..], [Step::Toggle { a: x, b: y }] if x == &a && y == &b));
    h.app.config.keys_mut()[0].steps.push(Step::Delay { ms: 400 });
    let output = h.warm(size);
    assert!(text_position(&output.shapes, "Étapes supplémentaires hors ON/OFF : elles s'exécutent à chaque appui.").is_some());
    assert!(matches!(&h.app.config.keys()[0].steps[..], [Step::Toggle { a: x, b: y }, Step::Delay { ms: 400 }] if x == &a && y == &b));
}
