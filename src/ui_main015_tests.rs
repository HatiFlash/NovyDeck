use super::*;

#[test]
fn main015_header_and_library_labels() {
    for lang in ["fr", "en"] {
        let mut h = Harness::new();
        crate::i18n::set_language(lang);
        h.app.preferences.language = lang.into();
        let size = vec2(1366.0, 768.0);
        h.app.gate.enabled = false;
        let paused = h.warm(size);
        let paused_label = if lang == "fr" { "Touches : en pause" } else { "Keys: paused" };
        let running_label = if lang == "fr" { "Touches : actives" } else { "Keys: running" };
        let pause = text_position(&paused.shapes, paused_label).expect("pause explicitly labeled");
        assert!(pause.y < 65.0, "status must stay on the header line: {pause:?}");
        h.app.gate.connected = true;
        h.app.gate.enabled = true;
        h.app.gate.dirty = false;
        let revision = h.app.gate.request_apply();
        assert!(h.app.gate.applied(revision));
        assert_eq!(h.app.gate.decision(), Decision::Execute);
        let running = h.warm(size);
        let run = text_position(&running.shapes, running_label).expect("running explicitly labeled");
        assert!(run.y < 65.0, "running status must stay on the header line: {run:?}");
        assert_eq!(h.app.gate.enabled, true);
        assert!(text_position(&running.shapes, if lang == "fr" { "Modèles" } else { "Templates" }).is_some());
        assert!(text_position(&running.shapes, if lang == "fr" { "Enregistrés" } else { "Saved" }).is_some());
        assert!(text_position(&running.shapes, "Mes binds").is_none());
    }
    crate::i18n::set_language("fr");
}

#[test]
fn main015_inspector_room_for_steps_and_appearance() {
    for (physical, dpi) in [(vec2(1366.0, 768.0), 1.0), (vec2(1920.0, 1080.0), 1.5)] {
        let mut h = demo_harness();
        let size = physical / dpi;
        h.ctx.set_pixels_per_point(dpi);
        h.app.selected = 0;
        h.app.config.keys_mut()[0].steps = vec![Step::Delay { ms: 100 }; 4];
        let out = h.warm(size);
        for i in 1..=2 {
            let marker = format!("{i:02}");
            let visible = text_position(&out.shapes, &marker).is_some();
            if !visible { eprintln!("step {i} not initially visible at {physical:?} @ {dpi}"); }
            assert!(visible, "step {i} at {physical:?} @ {dpi}");
        }
        h.ctx.data_mut(|d| d.insert_persisted(egui::Id::new("inspector_tab"), 1u8));
        let appearance = h.warm(size);
        let origin = text_position(&appearance.shapes, "Titre sur la touche").expect("top appearance control");
        let area = pos2(origin.x + 80.0, origin.y + 75.0);
        let mut reached = text_position(&appearance.shapes, "Taille de l’icône").is_some();
        for _ in 0..12 {
            if reached { break; }
            let frame = h.frame(size, vec![Input::PointerMoved(area), Input::MouseWheel { unit: egui::MouseWheelUnit::Point, delta: vec2(0.0, -90.0), modifiers: Modifiers::NONE }]);
            reached = text_position(&frame.shapes, "Taille de l’icône").is_some();
        }
        assert!(reached, "appearance controls unreachable by scrolling at {physical:?} @ {dpi}");
        assert_eq!(h.app.drop_targets.len(), 15);
    }
}

#[test]
fn main015_library_shows_five_complete_rows_at_laptop_size() {
    let mut h = Harness::new();
    let size = vec2(1366.0, 768.0);
    h.app.catalog_tab = 1;
    let frame = h.warm(size);
    let names: Vec<_> = catalog::presets().into_iter().take(5).map(|v| catalog::display_name(&v.key.label).to_owned()).collect();
    for name in &names {
        let visible = frame.shapes.iter().any(|shape| match &shape.shape {
            egui::Shape::Text(text) if text.galley.job.text == *name && text.pos.x > 1000.0 =>
                shape.clip_rect.contains_rect(egui::Rect::from_min_size(text.pos, text.galley.size())),
            _ => false,
        });
        assert!(visible, "library row {name:?} is clipped at 1366x768");
    }
}

#[test]
fn main015_empty_key_asks_assistant_without_running() {
    let mut h = Harness::new();
    let size = vec2(1366.0, 768.0);
    h.app.selected = 14;
    h.app.config.keys_mut()[14] = Key::default();
    let before = h.warm(size);
    let button = text_position(&before.shapes, "Demander à l’IA").expect("empty key assistant button");
    h.click(size, button);
    let after = h.warm(size);
    assert_eq!(h.app.selected, 14);
    assert!(h.app.show_workflow && h.app.workshop_chat.open);
    assert!(text_position(&after.shapes, "Assistant · Atelier").is_some());
    assert!(h.app.config.keys()[14].steps.is_empty());
    assert!(h.app.workshop_chat.pending.is_none() && h.app.workshop_chat.thinking.is_none());
    assert!(h.app.workshop_journal.confirm_real == false);
    assert!(!h.app.pending_chat_focus, "focus is consumed when the chat is rendered");
    h.frame(size, vec![egui::Event::Text("nouvelle demande".into())]);
    assert_eq!(h.app.workshop_chat.input, "nouvelle demande", "typing reaches the focused chat field");
    assert!(h.app.config.keys()[14].steps.is_empty(), "typing must not run a macro");
}

#[test]
fn main015_update_checkbox_remains_clickable() {
    let mut h = Harness::new();
    let size = vec2(1366.0, 768.0);
    h.app.settings = true;
    h.app.settings_tab = 0;
    let before = h.warm(size);
    let pos = text_position(&before.shapes, "Vérifier les mises à jour au démarrage").expect("update checkbox");
    let previous = h.app.preferences.update_check.unwrap_or(false);
    h.click(size, pos);
    assert_eq!(h.app.preferences.update_check, Some(!previous));
}

fn main015_capture(screen: &str, physical: Vec2, dpi: f32, dark: bool, lang: &str, out: &std::path::Path) {
    let size = physical / dpi;
    let mut h = demo_harness();
    crate::i18n::set_language(lang);
    h.app.preferences.language = lang.into();
    h.app.preferences.dark = dark;
    theme::apply(&h.ctx, dark);
    h.ctx.set_pixels_per_point(dpi);
    match screen {
        "deck" => h.app.selected = 8,
        "inspecteur-action" | "inspecteur-apparence" => {
            h.app.selected = 0;
            h.ctx.data_mut(|d| d.insert_persisted(egui::Id::new("inspector_tab"), if screen == "inspecteur-apparence" { 1u8 } else { 0u8 }));
        }
        "touche-vide" => {
            h.app.selected = 14;
            h.app.config.keys_mut()[14] = Key::default();
        }
        "reglages" => h.app.settings = true,
        _ => unreachable!(),
    }
    let frame = h.warm(size);
    let theme = if dark { "sombre" } else { "clair" };
    let name = format!("main015-{screen}-{}x{}-{theme}-{lang}.png", physical.x as u32, physical.y as u32);
    h.png(size, frame, &out.join(name));
}

#[test]
fn main015_captures() {
    let Some(out) = std::env::var_os("NOVYDECK_UI_OUTPUT") else { return; };
    let out = std::path::PathBuf::from(out);
    std::fs::create_dir_all(&out).unwrap();
    for (screen, physical, dpi, dark, lang) in [
        ("deck", vec2(1366.0, 768.0), 1.0, true, "fr"),
        ("deck", vec2(1366.0, 768.0), 1.0, false, "fr"),
        ("deck", vec2(1366.0, 768.0), 1.0, true, "en"),
        ("deck", vec2(1920.0, 1080.0), 1.5, true, "fr"),
        ("inspecteur-apparence", vec2(1366.0, 768.0), 1.0, true, "fr"),
        ("inspecteur-apparence", vec2(1366.0, 768.0), 1.0, false, "fr"),
        ("inspecteur-apparence", vec2(1366.0, 768.0), 1.0, true, "en"),
        ("reglages", vec2(1366.0, 768.0), 1.0, true, "fr"),
        ("reglages", vec2(1366.0, 768.0), 1.0, false, "fr"),
        ("reglages", vec2(1366.0, 768.0), 1.0, true, "en"),
        ("touche-vide", vec2(1366.0, 768.0), 1.0, true, "fr"),
        ("touche-vide", vec2(1366.0, 768.0), 1.0, false, "fr"),
    ] {
        let size = physical / dpi;
        let mut h = demo_harness();
        crate::i18n::set_language(lang);
        h.app.preferences.language = lang.into();
        h.app.preferences.dark = dark;
        theme::apply(&h.ctx, dark);
        h.ctx.set_pixels_per_point(dpi);
        match screen {
            "deck" => {
                h.app.selected = 8;
                h.app.gate.connected = true;
                h.app.gate.enabled = true;
                h.app.gate.dirty = false;
                let revision = h.app.gate.request_apply();
                assert!(h.app.gate.applied(revision));
                assert_eq!(h.app.gate.decision(), Decision::Execute);
            }
            "inspecteur-apparence" => {
                h.app.selected = 0;
                h.ctx.data_mut(|d| d.insert_persisted(egui::Id::new("inspector_tab"), 1u8));
            }
            "touche-vide" => {
                h.app.selected = 14;
                h.app.config.keys_mut()[14] = Key::default();
            }
            "reglages" => h.app.settings = true,
            _ => unreachable!(),
        }
        let frame = h.warm(size);
        let theme = if dark { "sombre" } else { "clair" };
        let name = format!("v015-main-{screen}-{}x{}-{theme}-{lang}.png", physical.x as u32, physical.y as u32);
        h.png(size, frame.clone(), &out.join(name));
        if screen == "deck" {
            assert!(text_position(&frame.shapes, if lang == "fr" { "Touches : actives" } else { "Keys: running" }).is_some(), "0.15 active key status missing");
        }
        if screen == "inspecteur-apparence" {
            assert!(text_position(&frame.shapes, if lang == "fr" { "Titre sur la touche" } else { "Key title" }).is_some(), "appearance inspector marker missing");
        }
        if screen == "reglages" {
            assert!(text_position(&frame.shapes, if lang == "fr" { "Vérifier les mises à jour au démarrage" } else { "Check for updates on startup" }).is_some(), "settings checkbox missing");
        }
    }
    crate::i18n::set_language("fr");
}
