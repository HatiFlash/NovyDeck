use super::*;
use egui::{vec2, Event as Input};

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
    assert!(text_position(&output.shapes, "Action 1").is_some());
    let action2 = text_position(&output.shapes, "Action 2").expect("onglet Action 2");
    h.click(size, action2);
    let output = h.warm(size);
    assert_eq!(h.app.config.keys()[0].label, "F9");
    assert!(h.app.config.keys()[0].state_display.is_none());
    let pos = text_position(&output.shapes, "Raccourci clavier").expect("raccourci clavier direct");
    h.click(size, pos);
    let output = h.warm(size);
    assert!(matches!(&h.app.config.keys()[0].steps[..], [Step::Toggle { b, .. }] if matches!(&b[..], [Step::Hotkey { .. }])), "steps after catalog click: {:?}, menu at {:?}", h.app.config.keys()[0].steps, pos);
    let branch = text_position(&output.shapes, "Action 2").expect("onglet Action 2 toujours visible");
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
fn onoff_existing_branches_and_extra_steps_are_not_silently_lost() {
    let mut h = Harness::new();
    let size = vec2(1366.0, 768.0);
    let a = vec![Step::Hotkey { keys: "F9".into() }, Step::Delay { ms: 125 }];
    let b = vec![Step::Hotkey { keys: "F10".into() }, Step::Delay { ms: 250 }];
    h.app.config.keys_mut()[0].steps = vec![Step::Toggle { a: a.clone(), b: b.clone() }];
    let output = h.warm(size);
    assert!(text_position(&output.shapes, "Action 1").is_some());
    assert!(text_position(&output.shapes, "Action 2").is_some());
    assert!(matches!(&h.app.config.keys()[0].steps[..], [Step::Toggle { a: x, b: y }] if x == &a && y == &b));
    h.app.config.keys_mut()[0].steps.push(Step::Delay { ms: 400 });
    let output = h.warm(size);
    assert!(text_position(&output.shapes, "Étapes supplémentaires hors ON/OFF : elles s'exécutent à chaque appui.").is_some());
    assert!(matches!(&h.app.config.keys()[0].steps[..], [Step::Toggle { a: x, b: y }, Step::Delay { ms: 400 }] if x == &a && y == &b));
}
