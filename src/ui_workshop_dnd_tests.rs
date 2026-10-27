// Glisser-déposer de l'Atelier : tout passe par des événements pointeur réels dans le Harness.
use super::*;
use egui::text_selection::LabelSelectionState;

const SLOT: &str = "Déposer ici";

fn delay(ms: u64) -> Step { Step::Delay { ms } }

fn scene(steps: Vec<Step>) -> (Harness, egui::Vec2) {
    let mut h = Harness::new();
    h.app.show_workflow = true;
    h.app.config.keys_mut()[0].steps = steps;
    (h, egui::vec2(1366.0, 768.0))
}

fn steps_of(h: &Harness) -> Vec<Step> { h.app.config.keys()[0].steps.clone() }

fn press(pos: egui::Pos2, pressed: bool) -> Input {
    Input::PointerButton { pos, button: PointerButton::Primary, pressed, modifiers: Default::default() }
}

fn dragging(h: &Harness) -> bool {
    h.ctx.data(|d| d.get_temp::<Vec<usize>>(egui::Id::new("workshop_drag")).is_some())
}

/// Appuie en `from`, passe par un point intermédiaire puis tient le pointeur en `to` (deux images
/// pour laisser apparaître l'emplacement). Le bouton reste enfoncé.
fn hold_drag(h: &mut Harness, size: egui::Vec2, from: egui::Pos2, to: egui::Pos2) -> egui::FullOutput {
    h.frame(size, vec![Input::PointerMoved(from)]);
    h.frame(size, vec![press(from, true)]);
    h.frame(size, vec![Input::PointerMoved(from + (to - from) * 0.5)]);
    h.frame(size, vec![Input::PointerMoved(to)]);
    assert!(dragging(h), "le glisser n'a pas démarré de {from:?} vers {to:?}");
    h.frame(size, vec![Input::PointerMoved(to)])
}

fn release(h: &mut Harness, size: egui::Vec2, at: egui::Pos2) -> egui::FullOutput {
    h.frame(size, vec![press(at, false)])
}

fn all_text(shapes: &[egui::epaint::ClippedShape], text: &str) -> Vec<egui::Pos2> {
    fn walk(shape: &egui::Shape, clip: egui::Rect, text: &str, out: &mut Vec<egui::Pos2>) {
        match shape {
            egui::Shape::Text(t) if t.galley.job.text == text => {
                let pos = t.pos + t.galley.size() * 0.5;
                if clip.contains(pos) { out.push(pos); }
            }
            egui::Shape::Vec(shapes) => for shape in shapes { walk(shape, clip, text, out); },
            _ => {}
        }
    }
    let mut out = Vec::new();
    for s in shapes { walk(&s.shape, s.clip_rect, text, &mut out); }
    out
}

fn at(shapes: &[egui::epaint::ClippedShape], text: &str) -> egui::Pos2 {
    text_position(shapes, text).unwrap_or_else(|| panic!("texte introuvable : {text}"))
}

#[test]
fn pressing_the_card_body_and_moving_drags_the_block_to_the_root_slot() {
    let (mut h, size) = scene(vec![delay(111), delay(222), delay(333)]);
    let out = h.warm(size);
    let body = at(&out.shapes, "Attendre 333 ms");
    let target = at(&out.shapes, "1");
    hold_drag(&mut h, size, body, target);
    release(&mut h, size, target);
    assert_eq!(steps_of(&h), vec![delay(333), delay(111), delay(222)]);
}

#[test]
fn block_goes_into_toggle_state_two_and_child_comes_back_to_root() {
    let (mut h, size) = scene(vec![Step::Toggle { a: vec![delay(1), delay(2)], b: vec![] }, delay(3)]);
    let out = h.warm(size);
    let source = at(&out.shapes, "Attendre 3 ms");
    let target = at(&out.shapes, "ÉTAT 2") + egui::vec2(8.0, 30.0);
    hold_drag(&mut h, size, source, target);
    release(&mut h, size, target);
    assert_eq!(steps_of(&h), vec![Step::Toggle { a: vec![delay(1), delay(2)], b: vec![delay(3)] }]);

    let (mut h, size) = scene(vec![delay(9), Step::Toggle { a: vec![delay(1), delay(2)], b: vec![] }]);
    let out = h.warm(size);
    let source = at(&out.shapes, "Attendre 1 ms");
    let target = at(&out.shapes, "1");
    hold_drag(&mut h, size, source, target);
    release(&mut h, size, target);
    assert_eq!(steps_of(&h), vec![delay(1), delay(9), Step::Toggle { a: vec![delay(2)], b: vec![] }]);
}

#[test]
fn exactly_one_drop_here_placeholder_appears_at_the_target_slot() {
    let (mut h, size) = scene(vec![delay(111), delay(222), delay(333)]);
    let out = h.warm(size);
    assert!(all_text(&out.shapes, SLOT).is_empty(), "pas d'emplacement hors glisser");
    let body = at(&out.shapes, "Attendre 333 ms");
    let target = at(&out.shapes, "2");
    let out = hold_drag(&mut h, size, body, target);
    let slots = all_text(&out.shapes, SLOT);
    assert_eq!(slots.len(), 1, "un seul emplacement attendu : {slots:?}");
    let (first, second) = (at(&out.shapes, "1").y, at(&out.shapes, "2").y);
    assert!(slots[0].y > first && slots[0].y < second, "emplacement entre les blocs 1 et 2 : {first} < {} < {second}", slots[0].y);
    release(&mut h, size, target);
    assert_eq!(steps_of(&h), vec![delay(111), delay(333), delay(222)]);
}

#[test]
fn empty_branch_is_a_visible_drop_zone_outside_and_during_drag() {
    let (mut h, size) = scene(vec![delay(7), Step::Repeat { count: 2, steps: vec![] }]);
    let out = h.warm(size);
    assert_eq!(all_text(&out.shapes, "Vide — glisser un bloc ici").len(), 1);
    let source = at(&out.shapes, "Attendre 7 ms");
    let target = at(&out.shapes, "RÉPÉTER") + egui::vec2(8.0, 34.0);
    let out = hold_drag(&mut h, size, source, target);
    assert_eq!(all_text(&out.shapes, SLOT).len(), 1, "la branche ciblée affiche Déposer ici");
    release(&mut h, size, target);
    assert_eq!(steps_of(&h), vec![Step::Repeat { count: 2, steps: vec![delay(7)] }]);
}

#[test]
fn cursor_is_grab_over_a_card_and_grabbing_while_dragging() {
    let (mut h, size) = scene(vec![delay(111), delay(222), delay(333)]);
    let out = h.warm(size);
    let body = at(&out.shapes, "Attendre 222 ms");
    h.frame(size, vec![Input::PointerMoved(body)]);
    let hover = h.frame(size, vec![Input::PointerMoved(body)]);
    assert_eq!(hover.platform_output.cursor_icon, egui::CursorIcon::Grab);
    let target = at(&out.shapes, "1");
    let held = hold_drag(&mut h, size, body, target);
    assert_eq!(held.platform_output.cursor_icon, egui::CursorIcon::Grabbing);
    release(&mut h, size, target);
}

#[test]
fn no_text_is_selected_by_pressing_and_dragging_a_card() {
    let (mut h, size) = scene(vec![delay(111), delay(222), delay(333)]);
    let out = h.warm(size);
    let body = at(&out.shapes, "Attendre 333 ms");
    let target = at(&out.shapes, "1");
    hold_drag(&mut h, size, body, target);
    assert!(!LabelSelectionState::load(&h.ctx).has_selection(), "sélection de texte pendant le glisser");
    release(&mut h, size, target);
    assert!(!LabelSelectionState::load(&h.ctx).has_selection(), "sélection de texte après le glisser");
}

#[test]
fn escape_cancels_a_body_drag_without_changing_anything() {
    let (mut h, size) = scene(vec![delay(111), delay(222), delay(333)]);
    let original = steps_of(&h);
    let out = h.warm(size);
    let body = at(&out.shapes, "Attendre 333 ms");
    let target = at(&out.shapes, "1");
    hold_drag(&mut h, size, body, target);
    h.frame(size, vec![Input::Key { key: egui::Key::Escape, physical_key: None, pressed: true, repeat: false, modifiers: Default::default() }]);
    assert!(!dragging(&h), "Échap doit annuler le glisser");
    release(&mut h, size, target);
    assert_eq!(steps_of(&h), original);
}

#[test]
fn dropping_a_container_into_its_own_child_is_refused() {
    let tree = vec![Step::Toggle { a: vec![delay(1), delay(2)], b: vec![] }, delay(3)];
    let (mut h, size) = scene(tree.clone());
    let out = h.warm(size);
    let source = at(&out.shapes, "1");
    let target = at(&out.shapes, "1.A1");
    let held = hold_drag(&mut h, size, source, target);
    assert_eq!(held.platform_output.cursor_icon, egui::CursorIcon::NotAllowed);
    assert!(all_text(&held.shapes, SLOT).is_empty(), "aucun emplacement dans sa propre descendance");
    release(&mut h, size, target);
    assert_eq!(steps_of(&h), tree);
}

#[test]
fn a_plain_click_on_the_card_still_selects_it() {
    let (mut h, size) = scene(vec![delay(111), delay(222), delay(333)]);
    let out = h.warm(size);
    h.click(size, at(&out.shapes, "Attendre 222 ms"));
    h.warm(size);
    assert_eq!(h.ctx.data(|d| d.get_temp::<Vec<usize>>(egui::Id::new("workshop_path"))), Some(vec![1]));
    assert!(!dragging(&h));
}

#[test]
fn library_drag_shows_the_same_single_placeholder() {
    let (mut h, size) = scene(vec![delay(111), delay(222)]);
    h.ctx.data_mut(|d| {
        d.insert_temp(egui::Id::new("workshop_palette_open"), true);
        d.insert_temp(egui::Id::new("workshop_search"), "Attendre".to_owned());
    });
    let out = h.warm(size);
    let source = out.shapes.iter().filter_map(|s| if let egui::Shape::Text(t) = &s.shape {
        let pos = t.pos + t.galley.size() * 0.5;
        (t.galley.job.text == crate::i18n::tr("Attendre") && pos.x < 260.0 && s.clip_rect.contains(pos)).then_some(pos)
    } else { None }).max_by(|a, b| a.x.total_cmp(&b.x)).expect("bloc Attendre de la bibliothèque");
    let target = at(&out.shapes, "2");
    h.frame(size, vec![Input::PointerMoved(source)]);
    h.frame(size, vec![press(source, true)]);
    h.frame(size, vec![Input::PointerMoved(target)]);
    h.frame(size, vec![Input::PointerMoved(target)]);
    let held = h.frame(size, vec![Input::PointerMoved(target)]);
    assert_eq!(all_text(&held.shapes, SLOT).len(), 1);
    h.frame(size, vec![press(target, false)]);
    assert_eq!(steps_of(&h).len(), 3);
}

#[test]
fn capture_drag_with_placeholder_and_ghost() {
    let Some(dir) = std::env::var_os("NOVYDECK_UI_OUTPUT") else { return };
    let dir = std::path::PathBuf::from(dir);
    std::fs::create_dir_all(&dir).unwrap();
    for lang in ["fr", "en"] {
        let (mut h, _) = scene(vec![
            delay(250), Step::Toggle { a: vec![delay(10), delay(20)], b: vec![] },
            Step::Repeat { count: 3, steps: vec![delay(40)] }, delay(500),
        ]);
        crate::i18n::set_language(lang);
        h.app.preferences.language = lang.into();
        h.ctx.set_pixels_per_point(1.25);
        let size = egui::vec2(1280.0, 800.0);
        let out = h.warm(size);
        let source = at(&out.shapes, "4");
        let target = at(&out.shapes, "1.A1") + egui::vec2(60.0, 0.0);
        let held = hold_drag(&mut h, size, source, target);
        assert_eq!(all_text(&held.shapes, crate::i18n::tr(SLOT)).len(), 1, "{lang}");
        h.png(size, held, &dir.join(format!("workshop-dnd-{lang}.png")));
        release(&mut h, size, target);
    }
    crate::i18n::set_language("fr");
}

#[test]
fn zz_probe_layers() {
    let mut lines = String::new();
    for y in [340.0_f32, 372.0, 378.0, 382.0, 386.0, 396.0] {
        let (mut h, size) = scene(vec![delay(111), delay(222), delay(333)]);
        let out = h.warm(size);
        let body = at(&out.shapes, "Attendre 222 ms");
        let pos = egui::pos2(body.x, y);
        h.frame(size, vec![Input::PointerMoved(pos)]);
        let f = h.frame(size, vec![press(pos, true)]);
        let layer = h.ctx.layer_id_at(pos);
        let layers: Vec<String> = h.ctx.memory(|m| m.areas().visible_layer_ids().into_iter()
            .map(|l| format!("{:?}:{:?}", l, m.area_rect(l.id))).collect());
        let cur = f.platform_output.cursor_icon;
        h.frame(size, vec![Input::PointerMoved(pos + egui::vec2(30.0, 30.0))]);
        h.frame(size, vec![Input::PointerMoved(pos + egui::vec2(60.0, 60.0))]);
        lines += &format!("[y={y} body={body:?} at={:?} cur={cur:?} drag={} egui_drag={:?} layers={layers:?}] ", layer, dragging(&h), h.ctx.dragged_id());
    }
    panic!("PROBEX {lines} ENDPROBE");
}
