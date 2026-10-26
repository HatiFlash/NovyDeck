use super::*;
use egui::{vec2, Event as Input};

fn menu_on_child_row(shapes: &[egui::epaint::ClippedShape], number: &str) -> egui::Pos2 {
    let child = text_position(shapes, number).expect("visible nested block number");
    fn collect(shape: &egui::Shape, clip: egui::Rect, out: &mut Vec<egui::Pos2>) {
        match shape {
            egui::Shape::Text(t) if t.galley.job.text == "..." => {
                let p = t.pos + t.galley.size() * 0.5;
                if clip.contains(p) { out.push(p); }
            }
            egui::Shape::Vec(shapes) => for shape in shapes { collect(shape, clip, out); },
            _ => {},
        }
    }
    let mut menus = Vec::new();
    for shape in shapes { collect(&shape.shape, shape.clip_rect, &mut menus); }
    menus.into_iter().find(|p| (p.y - child.y).abs() < 12.0).expect("child menu on same row")
}

#[test]
fn release0151_captures() {
    let Some(out) = std::env::var_os("NOVYDECK_UI_OUTPUT") else { return; };
    let out = std::path::PathBuf::from(out);
    std::fs::create_dir_all(&out).unwrap();
    let physical = vec2(1600.0, 1000.0);
    let dpi = 1.25;
    let size = physical / dpi;
    for lang in ["fr", "en"] {
        crate::i18n::set_language(lang);
        let mut h = demo_harness();
        crate::i18n::set_language(lang);
        h.app.preferences.language = lang.into();
        h.app.preferences.dark = true;
        theme::apply(&h.ctx, true);
        h.ctx.set_pixels_per_point(dpi);
        h.app.selected = 0;
        h.app.config.keys_mut()[0].label = "ON / OFF".into();
        h.app.config.keys_mut()[0].steps = vec![Step::Toggle {
            a: vec![Step::Hotkey { keys: "F9".into() }],
            b: vec![Step::Hotkey { keys: "F10".into() }],
        }];
        h.ctx.data_mut(|d| d.insert_persisted(egui::Id::new("inspector_tab"), 0u8));
        let frame = h.warm(size);
        h.click(size, text_position(&frame.shapes, "Action 2").expect("Action 2 tab"));
        let mut frame = h.warm(size);
        let branch = text_position(&frame.shapes, "Action 2").expect("Action 2 visible");
        for _ in 0..8 {
            if text_position(&frame.shapes, crate::i18n::tr("Enregistrer…")).is_some() { break; }
            frame = h.frame(size, vec![
                Input::PointerMoved(branch + egui::vec2(85.0, 40.0)),
                Input::MouseWheel { unit: egui::MouseWheelUnit::Point, delta: vec2(0.0, -55.0), modifiers: egui::Modifiers::NONE },
            ]);
        }
        assert!(text_position(&frame.shapes, "Action 1").is_some());
        assert!(text_position(&frame.shapes, "Action 2").is_some());
        assert!(text_position(&frame.shapes, "F10").is_some(), "Action 2 F10 visible in {lang}");
        assert!(text_position(&frame.shapes, crate::i18n::tr("Enregistrer…")).is_some(), "shortcut controls visible in {lang}");
        h.png(size, frame, &out.join(format!("v0151-onoff-{lang}.png")));

        crate::i18n::set_language(lang);
        let mut h = demo_harness();
        crate::i18n::set_language(lang);
        h.app.preferences.language = lang.into();
        h.app.preferences.dark = true;
        theme::apply(&h.ctx, true);
        h.ctx.set_pixels_per_point(dpi);
        h.app.show_workflow = true;
        h.app.selected = 0;
        h.app.config.keys_mut()[0].steps = vec![Step::Toggle {
            a: vec![Step::Hotkey { keys: "F9".into() }, Step::Delay { ms: 100 }],
            b: vec![Step::Hotkey { keys: "F10".into() }],
        }];
        let frame = h.warm(size);
        let menu = menu_on_child_row(&frame.shapes, "1.A1");
        h.click(size, menu);
        let frame = h.warm(size);
        assert!(text_position(&frame.shapes, crate::i18n::tr("Supprimer")).is_some(), "nested menu visible in {lang}");
        h.png(size, frame, &out.join(format!("v0151-workshop-{lang}.png")));
    }
    crate::i18n::set_language("fr");
}
