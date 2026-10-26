use super::*;

fn workshop() -> (Harness, egui::Vec2) {
    let mut h = Harness::new();
    h.app.show_workflow = true;
    h.app.config.keys_mut()[0].steps = vec![Step::Delay { ms: 111 }, Step::Delay { ms: 222 }, Step::Delay { ms: 333 }];
    (h, egui::vec2(1366.0, 768.0))
}

fn drag(h: &mut Harness, size: egui::Vec2, from: egui::Pos2, to: egui::Pos2, ctrl: bool) -> egui::FullOutput {
    let mods = egui::Modifiers { ctrl, ..Default::default() };
    let press = |pos, pressed| Input::PointerButton { pos, button: PointerButton::Primary, pressed, modifiers: mods };
    h.frame(size, vec![Input::PointerMoved(from), press(from, true)]);
    h.frame(size, vec![Input::PointerMoved(to)]);
    assert!(h.ctx.data(|d| d.get_temp::<Vec<usize>>(egui::Id::new("workshop_drag"))).is_some(), "drag never started from {from:?} to {to:?}");
    let pending = h.frame(size, vec![Input::PointerMoved(to)]);
    if let Some(dir) = std::env::var_os("NOVYDECK_UI_OUTPUT") {
        std::fs::create_dir_all(&dir).unwrap();
        h.png(size, pending, &std::path::PathBuf::from(dir).join("workshop-drag-pending.png"));
    }
    h.frame(size, vec![press(to, false)])
}

fn handles(shapes: &[egui::epaint::ClippedShape]) -> Vec<egui::Pos2> {
    let mut positions = Vec::new();
    for s in shapes {
        if let egui::Shape::Text(t) = &s.shape {
            if t.galley.job.text == "⋮⋮" {
                let pos = t.pos + t.galley.size() * 0.5;
                if s.clip_rect.contains(pos) { positions.push(pos); }
            }
        }
    }
    positions
}

#[test]
fn mouse_drag_places_block_before_first_and_duplicates_with_ctrl() {
    let (mut h, size) = workshop();
    let output = h.warm(size);
    let points = handles(&output.shapes);
    assert_eq!(points.len(), 3, "a handle per card");
    let released = drag(&mut h, size, points[2], points[0], false);
    assert!(matches!(h.app.config.keys()[0].steps.first(), Some(Step::Delay { ms: 333 })), "steps={:?} drop={:?} drag={:?} text={:?}", h.app.config.keys()[0].steps, h.ctx.data(|d| d.get_temp::<Vec<usize>>(egui::Id::new("workshop_drop_at"))), h.ctx.data(|d| d.get_temp::<Vec<usize>>(egui::Id::new("workshop_drag"))), text_position(&released.shapes, "⋮⋮"));
    let output = h.warm(size);
    let points = handles(&output.shapes);
    drag(&mut h, size, points[0], points[2], true);
    assert_eq!(h.app.config.keys()[0].steps.len(), 4);
    assert!(matches!(h.app.config.keys()[0].steps[2], Step::Delay { ms: 333 }));
}

#[test]
fn drag_into_then_else_repeat_and_back_out() {
    for branch in [0, 1, 2] {
        let (mut h, size) = workshop();
        let condition = Step::IfVariable { name: "x".into(), value: "1".into(), then_steps: vec![], else_steps: vec![] };
        let container = if branch == 2 { Step::Repeat { count: 2, steps: vec![] } } else { condition };
        h.app.config.keys_mut()[0].steps = vec![Step::Delay { ms: 111 }, container];
        let output = h.warm(size);
        let source = handles(&output.shapes)[0];
        let target = text_position(&output.shapes, if branch == 0 { "ALORS" } else if branch == 1 { "SINON" } else { "RÉPÉTER" }).unwrap();
        // The empty branch's slot follows its label.
        drag(&mut h, size, source, target + egui::vec2(5.0, 18.0), false);
        assert_eq!(h.app.config.keys()[0].steps.len(), 1, "branch {branch}");
        let output = h.warm(size);
        let source = handles(&output.shapes)[1];
        let destination = handles(&output.shapes)[0];
        drag(&mut h, size, source, destination, false);
        assert!(matches!(h.app.config.keys()[0].steps[0], Step::Delay { ms: 111 }), "branch {branch}");
    }
}

#[test]
fn escape_cancels_pending_drag_without_changing_order() {
    let (mut h, size) = workshop();
    let output = h.warm(size);
    let points = handles(&output.shapes);
    let original = h.app.config.keys()[0].steps.clone();
    let press = |pos, pressed| Input::PointerButton { pos, button: PointerButton::Primary, pressed, modifiers: Default::default() };
    h.frame(size, vec![Input::PointerMoved(points[2]), press(points[2], true)]);
    h.frame(size, vec![Input::PointerMoved(points[0])]);
    h.frame(size, vec![Input::Key { key: egui::Key::Escape, physical_key: None, pressed: true, repeat: false, modifiers: Default::default() }]);
    h.frame(size, vec![press(points[0], false)]);
    assert_eq!(h.app.config.keys()[0].steps, original);
}

#[test]
fn workshop_header_stays_one_line_at_compact_and_hidpi() {
    for (physical, dpi) in [(egui::vec2(1366.0, 768.0), 1.0), (egui::vec2(1920.0, 1080.0), 1.5)] {
        let (mut h, _) = workshop();
        h.ctx.set_pixels_per_point(dpi);
        let output = h.warm(physical / dpi);
        let top = egui::containers::panel::PanelState::load(&h.ctx, egui::Id::new("header")).unwrap().rect;
        let workshop_title = output.shapes.iter().filter_map(|s| if let egui::Shape::Text(t) = &s.shape {
            (t.galley.job.text.starts_with("Atelier · Touche")).then_some(t.pos.y)
        } else { None }).next().unwrap();
        let save = text_position(&output.shapes, "Enregistrer").unwrap().y;
        assert!(top.height() < 60.0, "global header wrapped at {physical:?} dpi={dpi}: {top:?}");
        assert!((save - workshop_title).abs() < 22.0, "workshop header wrapped at {physical:?} dpi={dpi}: {workshop_title} / {save}");
    }
}

#[test]
fn drag_reorder_can_be_undone_and_redone() {
    let (mut h, size) = workshop();
    let before = h.app.config.keys()[0].steps.clone();
    let positions = handles(&h.warm(size).shapes);
    drag(&mut h, size, positions[2], positions[0], false);
    let after = h.app.config.keys()[0].steps.clone();
    assert_ne!(after, before);
    let key = |key| Input::Key { key, physical_key: None, pressed: true, repeat: false, modifiers: egui::Modifiers { ctrl: true, ..Default::default() } };
    h.frame(size, vec![key(egui::Key::Z)]);
    assert_eq!(h.app.config.keys()[0].steps, before);
    h.frame(size, vec![key(egui::Key::Y)]);
    assert_eq!(h.app.config.keys()[0].steps, after);
}

fn seek_number(h: &mut Harness, size: egui::Vec2, number: &str) {
    let center = egui::pos2(size.x * 0.65, size.y * 0.52);
    navigation_seek(h, size, number, center, -90.0);
    assert!(text_position(&h.warm(size).shapes, number).is_some(), "number under the fold: {number}");
}

#[test]
fn branch_numbers_are_stable_in_french_and_english() {
    for lang in ["fr", "en"] {
        let (mut h, size) = workshop();
        crate::i18n::set_language(lang);
        h.app.preferences.language = lang.into();
        h.app.config.keys_mut()[0].steps = vec![
            Step::Delay { ms: 1 }, Step::Delay { ms: 2 },
            Step::IfVariable { name: "x".into(), value: "1".into(), then_steps: vec![Step::Delay { ms: 31 }], else_steps: vec![Step::Delay { ms: 32 }] },
            Step::Repeat { count: 2, steps: vec![Step::Delay { ms: 41 }] },
        ];
        for number in ["3.A1", "3.B1", "4.1"] { seek_number(&mut h, size, number); }
    }
    crate::i18n::set_language("fr");
}

fn palette_delay(shapes: &[egui::epaint::ClippedShape]) -> egui::Pos2 {
    shapes.iter().filter_map(|s| if let egui::Shape::Text(t) = &s.shape {
        let pos = t.pos + t.galley.size() * 0.5;
        (t.galley.job.text == crate::i18n::tr("Attendre") && pos.x < 260.0 && s.clip_rect.contains(pos)).then_some(pos)
    } else { None }).max_by(|a, b| a.x.total_cmp(&b.x)).expect("visible palette delay")
}

fn drag_palette(h: &mut Harness, size: egui::Vec2, source: egui::Pos2, destination: egui::Pos2) {
    let press = |pos, pressed| Input::PointerButton { pos, button: PointerButton::Primary, pressed, modifiers: Default::default() };
    h.frame(size, vec![Input::PointerMoved(source)]);
    h.frame(size, vec![press(source, true)]);
    h.frame(size, vec![Input::PointerMoved(destination)]);
    assert_eq!(h.ctx.data(|d| d.get_temp::<usize>(egui::Id::new("workshop_palette_drag"))), Some(5), "source {source:?} -> {destination:?}: palette drag not registered");
    h.frame(size, vec![Input::PointerMoved(destination)]);
    h.frame(size, vec![press(destination, false)]);
}

#[test]
fn palette_drag_inserts_at_specific_sibling_slot_and_inside_container() {
    for nested in [false, true] {
        let (mut h, size) = workshop();
        h.ctx.data_mut(|d| {
            d.insert_temp(egui::Id::new("workshop_palette_open"), true);
            d.insert_temp(egui::Id::new("workshop_search"), "Attendre".to_owned());
        });
        if nested {
            h.app.config.keys_mut()[0].steps = vec![Step::Repeat { count: 2, steps: vec![Step::Delay { ms: 222 }] }];
        }
        let output = h.warm(size);
        let source = palette_delay(&output.shapes);
        let destination = if nested {
            let child = text_position(&output.shapes, "1.1").expect("visible repeat child");
            egui::pos2(child.x, child.y)
        } else { handles(&output.shapes)[1] };
        drag_palette(&mut h, size, source, destination);
        let steps = &h.app.config.keys()[0].steps;
        if nested {
            assert!(matches!(&steps[0], Step::Repeat { steps, .. } if steps.len() == 2 && matches!(steps[0], Step::Delay { ms: 500 }) && matches!(steps[1], Step::Delay { ms: 222 })), "nested: {steps:?}");
        } else {
            assert_eq!(steps.len(), 4, "sibling: {steps:?}");
            assert!(matches!(steps[1], Step::Delay { ms: 500 }), "slot: {steps:?}");
            assert!(matches!(steps[2], Step::Delay { ms: 222 }));
        }
    }
}

#[test]
fn dragging_near_bottom_scrolls_workshop_list() {
    let (mut h, size) = workshop();
    h.app.config.keys_mut()[0].steps = (0..22).map(|i| Step::Delay { ms: i }).collect();
    let out = h.warm(size);
    let from = handles(&out.shapes)[0];
    let clip = h.ctx.data(|d| d.get_temp::<egui::Rect>(egui::Id::new("workshop_scroll_clip")))
        .expect("actual scroll viewport clip");
    let target = egui::pos2(from.x + 30.0, clip.bottom() - 8.0);
    assert!(clip.contains(target) && target.y > clip.bottom() - 48.0, "target outside bottom scroll edge: {target:?} / {clip:?}");
    let press = |pos, pressed| Input::PointerButton { pos, button: PointerButton::Primary, pressed, modifiers: Default::default() };
    h.frame(size, vec![Input::PointerMoved(from), press(from, true)]);
    h.frame(size, vec![Input::PointerMoved(target)]);
    assert!(h.ctx.data(|d| d.get_temp::<Vec<usize>>(egui::Id::new("workshop_drag"))).is_some());
    assert!(h.ctx.input(|i| i.pointer.primary_down() && i.pointer.hover_pos() == Some(target)), "drag pointer not held over edge");
    h.frame(size, vec![Input::PointerMoved(target)]);
    assert_eq!(h.ctx.data(|d| d.get_temp::<f32>(egui::Id::new("workshop_drag_scroll_offset"))), Some(18.0), "scroll edge never requested an offset; clip={clip:?} target={target:?}");
    let before = h.warm(size);
    for _ in 0..25 { h.frame(size, vec![Input::PointerMoved(target)]); }
    let after = h.warm(size);
    let last_visible = |shapes: &[egui::epaint::ClippedShape]| {
        (1..=22).rev().find(|i| text_position(shapes, &i.to_string()).is_some()).unwrap_or(0)
    };
    assert!(last_visible(&after.shapes) > last_visible(&before.shapes), "list did not scroll while dragging at bottom: {} -> {}", last_visible(&before.shapes), last_visible(&after.shapes));
    h.frame(size, vec![press(target, false)]);
}

#[test]
fn workshop_cards_show_branch_numbers_and_singular_summary() {
    let (mut h, size) = workshop();
    h.app.config.keys_mut()[0].steps = vec![Step::Repeat { count: 2, steps: vec![Step::Delay { ms: 42 }] }];
    let output = h.warm(size);
    assert!(text_position(&output.shapes, "1.1").is_some());
    assert!(text_position(&output.shapes, "Répéter 2 fois (1 bloc)").is_some());
}
