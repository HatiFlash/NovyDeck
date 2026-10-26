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
        let inspector = egui::containers::panel::PanelState::load(&h.ctx, egui::Id::new("key_inspector"))
            .expect("inspector panel").rect;
        fn step_in_inspector(shapes: &[egui::epaint::ClippedShape], marker: &str, inspector: Rect) -> bool {
            fn contains(shape: &egui::Shape, marker: &str, clip: Rect, inspector: Rect) -> bool {
                match shape {
                    egui::Shape::Text(t) if t.galley.job.text == marker => {
                        let center = t.pos + t.galley.size() * 0.5;
                        clip.contains(center) && inspector.contains(center) && center.x > inspector.center().x
                    }
                    egui::Shape::Vec(parts) => parts.iter().any(|part| contains(part, marker, clip, inspector)),
                    _ => false,
                }
            }
            shapes.iter().any(|shape| contains(&shape.shape, marker, shape.clip_rect, inspector))
        }
        for i in 1..=2 {
            let marker = format!("{i:02}");
            assert!(step_in_inspector(&out.shapes, &marker, inspector), "step {i} in inspector at {physical:?} @ {dpi}");
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
fn main015_captures_a() {
    let Some(out) = std::env::var_os("NOVYDECK_UI_OUTPUT") else { return; };
    let out = std::path::PathBuf::from(out);
    std::fs::create_dir_all(&out).unwrap();
    for screen in ["deck", "inspecteur-action", "inspecteur-apparence", "touche-vide", "reglages"] {
        main015_capture(screen, vec2(1366.0, 768.0), 1.0, true, "fr", &out);
        main015_capture(screen, vec2(1366.0, 768.0), 1.0, false, "fr", &out);
    }
    crate::i18n::set_language("fr");
}

#[test]
fn main015_captures_b() {
    let Some(out) = std::env::var_os("NOVYDECK_UI_OUTPUT") else { return; };
    let out = std::path::PathBuf::from(out);
    std::fs::create_dir_all(&out).unwrap();
    for screen in ["deck", "inspecteur-action", "inspecteur-apparence", "touche-vide", "reglages"] {
        main015_capture(screen, vec2(1366.0, 768.0), 1.0, true, "en", &out);
        main015_capture(screen, vec2(1920.0, 1080.0), 1.5, true, "fr", &out);
    }
    crate::i18n::set_language("fr");
}
