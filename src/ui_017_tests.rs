use super::*;

#[test]
fn ui017_deck_color_saved_and_legacy_black() {
    let old: crate::catalog::Preferences = serde_json::from_str("{}").unwrap();
    assert_eq!(old.deck_color, None);
    assert!(!old.activity_collapsed);
    let mut h = Harness::new();
    crate::i18n::set_language("fr");
    let size = vec2(1366.0, 768.0);
    h.app.settings = true;
    let frame = h.warm(size);
    assert!(text_position(&frame.shapes, "Couleur du deck affiché").is_some());
    let combo = text_position(&frame.shapes, "Noir actuel").unwrap();
    h.click(size, combo);
    let opened = h.warm(size);
    let white = text_position(&opened.shapes, "Blanc").expect("white option");
    h.click(size, white);
    assert_eq!(h.app.preferences.deck_color, Some([237, 239, 240]));
    let saved: crate::catalog::Preferences = serde_json::from_slice(&std::fs::read(h.dir.path().join("preferences.json")).unwrap()).unwrap();
    assert_eq!(saved.deck_color, h.app.preferences.deck_color);
    assert!(crate::deck_chassis::light(saved.deck_color));
    crate::i18n::set_language("fr");
}

#[test]
fn ui017_preview_grows_without_clipping_at_three_sizes() {
    let mut widths = Vec::new();
    for (physical, dpi) in [(vec2(1060.0, 760.0), 1.0), (vec2(1366.0, 768.0), 1.25), (vec2(1920.0, 1080.0), 1.0)] {
        let mut h = Harness::new();
        let size = physical / dpi;
        h.ctx.set_pixels_per_point(dpi);
        let _frame = h.warm(size);
        assert_eq!(h.app.drop_targets.len(), 15);
        let first = h.app.drop_targets[0];
        let last = h.app.drop_targets[14];
        assert!(first.min.x >= 0.0 && last.max.x <= size.x && last.max.y < size.y - 125.0, "deck clipped at {physical:?} @ {dpi}: {last:?}");
        widths.push(first.width() * dpi);
    }
    assert!(widths[2] > widths[0] * 1.25, "preview must grow: {widths:?}");
}

#[test]
fn ui017_activity_collapses_and_expands_with_persistence() {
    let mut h = Harness::new();
    crate::i18n::set_language("fr");
    let size = vec2(1366.0, 768.0);
    h.app.show_logs = true;
    h.app.logs.push_back("Événement témoin".into());
    let expanded = h.warm(size);
    assert!(text_position(&expanded.shapes, "Événement témoin").is_some());
    let toggle = text_position(&expanded.shapes, "▾ Activité récente").expect("activity header");
    h.click(size, toggle);
    assert!(h.app.preferences.activity_collapsed);
    let collapsed = h.warm(size);
    assert!(text_position(&collapsed.shapes, "▸ Activité récente").is_some());
    assert!(collapsed.shapes.iter().any(|s| matches!(&s.shape, egui::Shape::Text(t) if t.galley.job.text.contains("Événement témoin"))));
    let saved: crate::catalog::Preferences = serde_json::from_slice(&std::fs::read(h.dir.path().join("preferences.json")).unwrap()).unwrap();
    assert!(saved.activity_collapsed);
    let toggle = text_position(&collapsed.shapes, "▸ Activité récente").unwrap();
    h.click(size, toggle);
    assert!(!h.app.preferences.activity_collapsed);
    crate::i18n::set_language("fr");
}

#[test]
fn ui017_captures() {
    let Some(out) = std::env::var_os("NOVYDECK_UI_OUTPUT") else { return; };
    let out = std::path::PathBuf::from(out);
    std::fs::create_dir_all(&out).unwrap();
    for (name, color, physical, collapsed) in [
        ("noir-1060", None, vec2(1060.0, 760.0), false),
        ("blanc-1920", Some([237, 239, 240]), vec2(1920.0, 1080.0), false),
        ("libre-1920", Some([95, 55, 165]), vec2(1920.0, 1080.0), false),
        ("activite-repliee", Some([35, 64, 110]), vec2(1366.0, 768.0), true),
    ] {
        let size = physical;
        let mut h = Harness::new();
        crate::i18n::set_language("fr");
        h.app.preferences.deck_color = color;
        if collapsed { h.app.show_logs = true; h.app.preferences.activity_collapsed = true; h.app.logs.push_back("Événement témoin".into()); }
        let frame = h.warm(size);
        h.png(size, frame, &out.join(format!("ui017-{name}.png")));
    }
}
