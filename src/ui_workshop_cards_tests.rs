// Cartes et branches de l'Atelier (tranche 2) : catégories, repli, chips, désactivé, hauteur, clip.
use super::*;
use egui::Event as Input;

fn delay(ms: u64) -> Step { Step::Delay { ms } }

fn scene(steps: Vec<Step>) -> Harness {
    let mut h = Harness::new();
    h.app.show_workflow = true;
    h.app.config.keys_mut()[0].steps = steps;
    h
}

fn steps_of(h: &Harness) -> Vec<Step> { h.app.config.keys()[0].steps.clone() }

fn toggle_scene() -> Vec<Step> {
    vec![Step::Toggle { a: vec![delay(111)], b: vec![delay(222)] }, delay(9)]
}

fn fold_id(path: &[usize]) -> egui::Id { egui::Id::new(("workshop_fold", path.to_vec())) }
fn set_fold(h: &Harness, path: &[usize], collapsed: bool) { h.ctx.data_mut(|d| d.insert_temp(fold_id(path), collapsed)); }
fn is_folded(h: &Harness, path: &[usize]) -> bool { h.ctx.data(|d| d.get_temp::<bool>(fold_id(path))).unwrap_or(false) }

fn press(pos: egui::Pos2, pressed: bool) -> Input {
    Input::PointerButton { pos, button: egui::PointerButton::Primary, pressed, modifiers: Default::default() }
}

fn dragging(h: &Harness) -> bool {
    h.ctx.data(|d| d.get_temp::<Vec<usize>>(egui::Id::new("workshop_drag")).is_some())
}

fn hold_drag(h: &mut Harness, size: egui::Vec2, from: egui::Pos2, to: egui::Pos2) -> egui::FullOutput {
    h.frame(size, vec![Input::PointerMoved(from)]);
    h.frame(size, vec![press(from, true)]);
    h.frame(size, vec![Input::PointerMoved(from + (to - from) * 0.5)]);
    h.frame(size, vec![Input::PointerMoved(to)]);
    assert!(dragging(h), "le glisser n'a pas démarré de {from:?} vers {to:?}");
    h.frame(size, vec![Input::PointerMoved(to)])
}

fn at(shapes: &[egui::epaint::ClippedShape], text: &str) -> egui::Pos2 {
    text_position(shapes, text).unwrap_or_else(|| panic!("texte introuvable : {text}"))
}

fn walk<'a>(shape: &'a egui::Shape, f: &mut impl FnMut(&'a egui::Shape)) {
    f(shape);
    if let egui::Shape::Vec(v) = shape { for s in v { walk(s, f); } }
}

fn count_fill(shapes: &[egui::epaint::ClippedShape], fill: Color32) -> usize {
    let mut n = 0;
    for s in shapes { walk(&s.shape, &mut |s| if matches!(s, egui::Shape::Rect(r) if r.fill == fill) { n += 1; }); }
    n
}

fn count_texture(shapes: &[egui::epaint::ClippedShape], id: egui::TextureId) -> usize {
    let mut n = 0;
    for s in shapes { walk(&s.shape, &mut |s| if matches!(s, egui::Shape::Mesh(m) if m.texture_id == id) { n += 1; }); }
    n
}

fn text_fade(shapes: &[egui::epaint::ClippedShape], text: &str) -> f32 {
    let mut found = None;
    for s in shapes { walk(&s.shape, &mut |s| if let egui::Shape::Text(t) = s { if t.galley.job.text == text && found.is_none() { found = Some(t.opacity_factor * t.fallback_color.a() as f32 / 255.0); } }); }
    found.unwrap_or_else(|| panic!("galley introuvable : {text}"))
}

fn card_rect(h: &Harness, path: &[usize]) -> egui::Rect {
    h.ctx.data(|d| d.get_temp::<egui::Rect>(egui::Id::new(("workshop_card_rect", path.to_vec())))).expect("rectangle de carte")
}

/// Centre du chevron dessiné (tracé à 3 points, trait 1,8).
fn chevron(shapes: &[egui::epaint::ClippedShape]) -> Option<egui::Pos2> {
    let mut found = None;
    for s in shapes {
        walk(&s.shape, &mut |s| if let egui::Shape::Path(p) = s {
            if p.points.len() == 3 && (p.stroke.width - 1.8).abs() < 0.01 && found.is_none() { found = Some(egui::Rect::from_points(&p.points).center()); }
        });
    }
    found
}

#[test]
fn category_colour_and_icon_are_identical_on_card_and_library() {
    let size = egui::vec2(1366.0, 768.0);
    // (type de bloc de la bibliothèque, nom affiché pour la recherche)
    for (kind, name) in [(0usize, "Raccourci clavier"), (5, "Attendre"), (2, "Application"), (4, "Média / volume"), (15, "Répéter des blocs"), (39, "Si… sinon")] {
        let step = crate::automation::step(kind);
        let mut h = scene(vec![step.clone()]);
        h.ctx.data_mut(|d| {
            d.insert_temp(egui::Id::new("workshop_palette_open"), true);
            d.insert_temp(egui::Id::new("workshop_search"), crate::i18n::tr(name).to_owned());
        });
        let out = h.warm(size);
        let style = extras::block_style(&h.ctx, &step);
        let fill = extras::badge_fill(style.color);
        assert!(count_fill(&out.shapes, fill) >= 2, "{name} : pastille {fill:?} attendue sur la carte ET dans la bibliothèque");
        let texture = crate::iconset::texture(&h.ctx, style.ico, style.family, extras::BADGE_ICON).expect("icône rasterisée");
        assert!(count_texture(&out.shapes, texture.id()) >= 2, "{name} : même icône sur la carte et dans la bibliothèque");
    }
    // Les catégories demandées ont des couleurs distinctes.
    let h = scene(vec![]);
    let colors: Vec<Color32> = [
        Step::Hotkey { keys: "A".into() }, Step::Media { command: "play_pause".into() }, Step::Launch { program: "obs64.exe".into(), args: vec![], focus_if_running: true },
        crate::automation::step(46), delay(1), crate::automation::step(14), crate::automation::step(15), crate::automation::step(36), Step::PageNext,
    ].iter().map(|s| extras::block_style(&h.ctx, s).color).collect();
    for (i, a) in colors.iter().enumerate() { for b in &colors[i + 1..] { assert_ne!(a, b, "deux catégories partagent une couleur : {colors:?}"); } }
}

#[test]
fn collapsing_a_toggle_hides_children_shows_block_count_and_survives_frames() {
    let size = egui::vec2(1366.0, 768.0);
    let mut h = scene(toggle_scene());
    let out = h.warm(size);
    assert!(text_position(&out.shapes, "Attendre 111 ms").is_some() && text_position(&out.shapes, "Attendre 222 ms").is_some());
    assert!(text_position(&out.shapes, "2 blocs").is_none(), "pas de résumé tant que le conteneur est déplié");
    set_fold(&h, &[0], true);
    for _ in 0..3 {
        let out = h.frame(size, vec![]);
        assert!(text_position(&out.shapes, "Attendre 111 ms").is_none() && text_position(&out.shapes, "Attendre 222 ms").is_none(), "enfants masqués");
        assert!(text_position(&out.shapes, "2 blocs").is_some(), "résumé « 2 blocs » une fois replié");
        assert!(text_position(&out.shapes, "Attendre 9 ms").is_some(), "le bloc suivant reste visible");
    }
    set_fold(&h, &[0], false);
    let out = h.warm(size);
    assert!(text_position(&out.shapes, "Attendre 111 ms").is_some() && text_position(&out.shapes, "2 blocs").is_none());
}

#[test]
fn the_drawn_chevron_folds_and_unfolds_the_container() {
    let size = egui::vec2(1366.0, 768.0);
    let mut h = scene(toggle_scene());
    let out = h.warm(size);
    h.click(size, chevron(&out.shapes).expect("chevron dessiné"));
    let out = h.warm(size);
    assert!(is_folded(&h, &[0]) && text_position(&out.shapes, "Attendre 111 ms").is_none() && text_position(&out.shapes, "2 blocs").is_some());
    assert!(h.ctx.data(|d| d.get_temp::<Vec<usize>>(egui::Id::new("workshop_path"))).is_none(), "le chevron ne sélectionne pas la carte");
    h.click(size, chevron(&out.shapes).expect("chevron replié"));
    let out = h.warm(size);
    assert!(!is_folded(&h, &[0]) && text_position(&out.shapes, "Attendre 111 ms").is_some());
}

#[test]
fn branch_chips_are_rendered_in_french_and_english() {
    // Fenêtre haute : tout l'arbre est visible (egui ne peint pas les libellés hors de la zone défilante).
    let size = egui::vec2(1366.0, 1400.0);
    let tree = vec![
        Step::Toggle { a: vec![delay(1)], b: vec![delay(2)] },
        Step::IfWindow { title: "Zed".into(), then_steps: vec![delay(3)], else_steps: vec![delay(4)] },
        Step::Repeat { count: 3, steps: vec![delay(5)] },
    ];
    for (lang, expected) in [
        ("fr", ["ÉTAT 1", "· ON", "ÉTAT 2", "· OFF", "ALORS", "SINON", "RÉPÉTER", "×3", "1 bloc"]),
        ("en", ["STATE 1", "· ON", "STATE 2", "· OFF", "THEN", "ELSE", "REPEAT", "×3", "1 block"]),
    ] {
        let mut h = scene(tree.clone());
        crate::i18n::set_language(lang);
        h.app.preferences.language = lang.into();
        let out = h.warm(size);
        for text in expected { assert!(text_position(&out.shapes, text).is_some(), "{lang} : « {text} » absent"); }
    }
    crate::i18n::set_language("fr");
}

#[test]
fn a_disabled_card_is_rendered_with_reduced_opacity() {
    let size = egui::vec2(1366.0, 768.0);
    let mut h = scene(vec![delay(100), Step::Disabled { step: Box::new(delay(700)) }]);
    let out = h.warm(size);
    let (on, off) = (text_fade(&out.shapes, "Attendre 100 ms"), text_fade(&out.shapes, "Attendre 700 ms"));
    assert_eq!(on, 1.0);
    assert!(off < 0.7, "carte désactivée grisée : opacité {off}");
}

#[test]
fn a_simple_card_is_compact_and_keeps_its_text_inside() {
    for ppp in [1.0f32, 1.25] {
        let mut h = scene(vec![Step::Hotkey { keys: "CTRL+C".into() }, delay(250)]);
        h.ctx.set_pixels_per_point(ppp);
        let size = egui::vec2(1366.0, 768.0) / ppp;
        let out = h.warm(size);
        let r = card_rect(&h, &[0]);
        assert!(r.height() <= 56.0 && r.height() >= 36.0, "hauteur de carte simple à @{ppp} : {}", r.height());
        let mut texts = Vec::new();
        for s in &out.shapes { walk(&s.shape, &mut |t| if let egui::Shape::Text(t) = t { texts.push(egui::Rect::from_min_size(t.pos, t.galley.size())); }); }
        let summary = texts.iter().filter(|t| r.contains(t.center())).collect::<Vec<_>>();
        assert!(summary.len() >= 3, "numéro, titre, résumé attendus dans la carte : {summary:?}");
        for t in summary { assert!(r.expand(1.0).contains_rect(*t) || t.max.x <= r.max.x + 40.0 && t.min.y >= r.min.y - 1.0 && t.max.y <= r.max.y + 1.0, "texte hors carte {t:?} / {r:?}"); }
    }
}

#[test]
fn no_text_overlaps_or_leaves_its_clip_at_1366_by_768_at_125_percent() {
    let long = "https://exemple.org/un/chemin/vraiment/tres/long/pour/forcer/la/troncature/du/resume/sur/une/seule/ligne".to_owned();
    let tree = vec![
        Step::Url { url: long.clone() },
        Step::Toggle { a: vec![Step::Url { url: long.clone() }, Step::Hotkey { keys: "CTRL+SHIFT+ALT+F12".into() }], b: vec![Step::Repeat { count: 12, steps: vec![Step::Url { url: long }] }] },
        Step::IfWindow { title: "Une fenêtre avec un titre extrêmement long qui dépasse la largeur".into(), then_steps: vec![delay(10)], else_steps: vec![] },
    ];
    let mut h = scene(tree);
    h.ctx.set_pixels_per_point(1.25);
    let size = egui::vec2(1366.0, 768.0) / 1.25;
    let out = h.warm(size);
    let clip = h.ctx.data(|d| d.get_temp::<egui::Rect>(egui::Id::new("workshop_scroll_clip"))).expect("zone de l'Atelier");
    let mut galleys: Vec<(egui::Rect, String)> = Vec::new();
    for s in &out.shapes {
        walk(&s.shape, &mut |t| if let egui::Shape::Text(t) = t {
            let rect = egui::Rect::from_min_size(t.pos, t.galley.size());
            if clip.contains(rect.center()) { galleys.push((rect, t.galley.job.text.clone())); }
        });
    }
    assert!(galleys.len() > 12, "l'Atelier doit être rendu : {}", galleys.len());
    for (rect, text) in &galleys {
        assert!(clip.expand(1.0).contains_rect(*rect), "« {text} » déborde de la zone {clip:?} : {rect:?}");
    }
    for (i, (a, ta)) in galleys.iter().enumerate() {
        for (b, tb) in &galleys[i + 1..] {
            assert!(!a.shrink(0.75).intersects(b.shrink(0.75)), "chevauchement « {ta} » {a:?} / « {tb} » {b:?}");
        }
    }
    // Chaque carte tient dans la largeur disponible.
    // (la hauteur peut dépasser la zone visible : la liste défile ; seule la largeur est contrainte)
    for path in [vec![0usize], vec![1], vec![2]] {
        let card = card_rect(&h, &path);
        assert!(card.min.x >= clip.min.x - 1.0 && card.max.x <= clip.max.x + 1.0, "carte {path:?} {card:?} déborde en largeur de {clip:?}");
    }
}

#[test]
fn dropping_on_a_collapsed_container_appends_to_its_first_branch() {
    let size = egui::vec2(1366.0, 768.0);
    let mut h = scene(toggle_scene());
    set_fold(&h, &[0], true);
    let out = h.warm(size);
    let source = at(&out.shapes, "Attendre 9 ms");
    let target = at(&out.shapes, "2 blocs");
    hold_drag(&mut h, size, source, target);
    h.frame(size, vec![press(target, false)]);
    assert_eq!(steps_of(&h), vec![Step::Toggle { a: vec![delay(111), delay(9)], b: vec![delay(222)] }]);

    let mut h = scene(vec![Step::Repeat { count: 2, steps: vec![delay(1)] }, delay(9)]);
    set_fold(&h, &[0], true);
    let out = h.warm(size);
    let (source, target) = (at(&out.shapes, "Attendre 9 ms"), at(&out.shapes, "1 bloc"));
    hold_drag(&mut h, size, source, target);
    h.frame(size, vec![press(target, false)]);
    assert_eq!(steps_of(&h), vec![Step::Repeat { count: 2, steps: vec![delay(1), delay(9)] }]);
}

#[test]
fn hovering_a_collapsed_container_while_dragging_unfolds_it_after_a_delay() {
    let size = egui::vec2(1366.0, 768.0);
    let mut h = scene(toggle_scene());
    set_fold(&h, &[0], true);
    let out = h.warm(size);
    let source = at(&out.shapes, "Attendre 9 ms");
    let target = at(&out.shapes, "2 blocs");
    hold_drag(&mut h, size, source, target);
    for _ in 0..4 { h.frame(size, vec![Input::PointerMoved(target)]); }
    assert!(is_folded(&h, &[0]), "pas de dépliage avant ~600 ms");
    for _ in 0..16 { h.frame(size, vec![Input::PointerMoved(target)]); }
    assert!(!is_folded(&h, &[0]), "dépliage après un survol prolongé");
    h.frame(size, vec![Input::Key { key: egui::Key::Escape, physical_key: None, pressed: true, repeat: false, modifiers: Default::default() }]);
    h.frame(size, vec![press(target, false)]);
    assert!(!dragging(&h));
}

#[test]
fn capture_workshop_cards_in_both_languages_and_themes() {
    let Some(dir) = std::env::var_os("NOVYDECK_UI_OUTPUT") else { return };
    let dir = std::path::PathBuf::from(dir);
    std::fs::create_dir_all(&dir).unwrap();
    let folder = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/counter.sdPlugin");
    let plugin = crate::plugins::parse(&folder, &std::fs::read(folder.join("manifest.json")).unwrap()).unwrap();
    let action = plugin.actions[0].uuid.clone();
    for lang in ["fr", "en"] {
        for dark in [true, false] {
            let steps = vec![
                Step::Hotkey { keys: "CTRL+SHIFT+M".into() },
                Step::Toggle {
                    a: vec![Step::Launch { program: "obs64.exe".into(), args: vec![], focus_if_running: true }, delay(250)],
                    b: vec![Step::Url { url: "https://novydeck.app/aide".into() }, Step::Hotkey { keys: "ALT+TAB".into() }],
                },
                Step::IfWindow { title: "Discord".into(), then_steps: vec![Step::Hotkey { keys: "CTRL+K".into() }], else_steps: vec![delay(500)] },
                Step::Repeat { count: 3, steps: vec![Step::Hotkey { keys: "VOLUME_UP".into() }, delay(120)] },
                Step::PluginAction { plugin: plugin.uuid.clone(), action: action.clone(), settings: serde_json::json!({}), label: "Compteur".into() },
                Step::Disabled { step: Box::new(Step::Notify { title: "Terminé".into(), message: "La macro est finie".into() }) },
            ];
            let mut h = scene(steps);
            h.app.plugins = Some(crate::plugin_runtime::Manager::new(vec![plugin.clone()]));
            crate::i18n::set_language(lang);
            h.app.preferences.language = lang.into();
            h.app.preferences.dark = dark;
            theme::apply(&h.ctx, dark);
            h.ctx.set_pixels_per_point(1.25);
            let size = egui::vec2(1280.0, 800.0);
            h.warm(size);
            h.ctx.data_mut(|d| d.insert_temp(egui::Id::new("workshop_path"), vec![2_usize]));
            let out = h.warm(size);
            let clip = h.ctx.data(|d| d.get_temp::<egui::Rect>(egui::Id::new("workshop_scroll_clip"))).expect("zone de l'Atelier");
            // La séquence est plus haute que l'écran (défilement normal) : la capture montre son début,
            // la première carte doit être entièrement visible dans la zone de l'Atelier.
            assert!(clip.contains_rect(card_rect(&h, &[0])), "{lang}/{dark} : première carte visible clip={clip:?}");
            h.png(size, out, &dir.join(format!("workshop-cards-{lang}-{}.png", if dark { "dark" } else { "light" })));
        }
    }
    crate::i18n::set_language("fr");
    theme::apply(&egui::Context::default(), true);
}
