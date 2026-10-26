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
fn workshop_cards_show_branch_numbers_and_singular_summary() {
    let (mut h, size) = workshop();
    h.app.config.keys_mut()[0].steps = vec![Step::Repeat { count: 2, steps: vec![Step::Delay { ms: 42 }] }];
    let output = h.warm(size);
    assert!(text_position(&output.shapes, "1.1").is_some());
    assert!(text_position(&output.shapes, "Répéter 2 fois (1 bloc)").is_some());
}
