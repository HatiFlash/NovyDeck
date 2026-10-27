// Bibliothèque de blocs et plugins de l'Atelier (tranche 3) : recherche, favoris, double-clic, plugins groupés,
// carte d'action de plugin, plugin manquant, inventaire en cache. Tout passe par des événements réels du Harness.
use super::*;
use egui::Event as Input;

const NO_RESULT: &str = "Aucun bloc trouvé. Essaie un autre mot-clé.";

fn size() -> egui::Vec2 { vec2(1366.0, 768.0) }
fn tall() -> egui::Vec2 { vec2(1600.0, 1100.0) }

fn delay(ms: u64) -> Step { Step::Delay { ms } }

fn steps_of(h: &Harness) -> Vec<Step> { h.app.config.keys()[0].steps.clone() }

fn press(pos: egui::Pos2, pressed: bool) -> Input {
    Input::PointerButton { pos, button: egui::PointerButton::Primary, pressed, modifiers: Default::default() }
}

fn at(shapes: &[egui::epaint::ClippedShape], text: &str) -> egui::Pos2 {
    text_position(shapes, text).unwrap_or_else(|| panic!("texte introuvable : {text}"))
}

fn visible(shapes: &[egui::epaint::ClippedShape], text: &str) -> bool { text_position(shapes, text).is_some() }

/// Plugin dérivé de la fixture `counter.sdPlugin` avec `actions` actions nommées « <name> action <n> ».
fn plugin(name: &str, uuid: &str, actions: usize) -> crate::plugins::Plugin {
    let folder = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/counter.sdPlugin");
    let mut plugin = crate::plugins::parse(&folder, &std::fs::read(folder.join("manifest.json")).unwrap()).unwrap();
    plugin.name = name.into();
    plugin.uuid = uuid.into();
    plugin.prerequisite = None;
    plugin.inspector = Some("inspector.html".into()); // Offline routing fixture; no WebView is asserted.
    let template = plugin.actions[0].clone();
    plugin.actions = (1..=actions).map(|n| {
        let mut action = template.clone();
        action.uuid = format!("{uuid}.action{n}");
        action.name = format!("{name} action {n}");
        action
    }).collect();
    plugin
}

fn scene(steps: Vec<Step>, plugins: Vec<crate::plugins::Plugin>) -> Harness {
    crate::i18n::set_language("fr");
    let mut h = Harness::new();
    h.app.show_workflow = true;
    h.app.config.keys_mut()[0].steps = steps;
    if !plugins.is_empty() { h.app.plugins = Some(crate::plugin_runtime::Manager::new(plugins)); }
    h.ctx.data_mut(|d| d.insert_temp(egui::Id::new("workshop_palette_open"), true));
    h
}

fn set_search(h: &Harness, text: &str) { h.ctx.data_mut(|d| d.insert_temp(egui::Id::new("workshop_search"), text.to_owned())); }

fn rect_of(h: &Harness, kind: &str, salt: &str) -> egui::Rect {
    h.ctx.data(|d| d.get_temp::<egui::Rect>(egui::Id::new((kind.to_owned(), salt.to_owned())))).unwrap_or_else(|| panic!("{kind} {salt} introuvable"))
}

/// Clique le champ de recherche puis tape `text` au clavier.
fn type_in_search(h: &mut Harness, text: &str) -> egui::FullOutput {
    let out = h.warm(size());
    let field = at(&out.shapes, "Rechercher un bloc");
    h.click(size(), field);
    h.frame(size(), vec![Input::Text(text.into())]);
    h.warm(size())
}

#[test]
fn typed_search_finds_wait_by_french_alias_and_hides_other_blocks() {
    for query in ["attente", "WAIT", "ATTENDRE"] {
        let mut h = scene(vec![delay(100)], vec![]);
        let out = h.warm(size());
        assert!(visible(&out.shapes, "Saisir du texte") || visible(&out.shapes, "Clavier"), "sans recherche, les catégories s'affichent");
        let out = type_in_search(&mut h, query);
        assert_eq!(h.ctx.data(|d| d.get_temp::<String>(egui::Id::new("workshop_search"))).as_deref(), Some(query), "le clavier alimente la recherche");
        assert!(visible(&out.shapes, "Attendre"), "« {query} » doit trouver le bloc Attendre");
        assert!(!visible(&out.shapes, "Saisir du texte"), "« {query} » doit masquer les autres blocs");
        assert!(!visible(&out.shapes, "Application"), "« {query} » doit masquer les autres blocs");
        assert!(!visible(&out.shapes, NO_RESULT));
    }
}

#[test]
fn search_ignores_accents_and_case_and_matches_english_names() {
    let mut h = scene(vec![delay(100)], vec![]);
    for query in ["fenetre", "FENÊTRE", "Fenêtre"] {
        set_search(&h, query);
        let out = h.warm(size());
        assert!(visible(&out.shapes, "Attendre une fenêtre"), "« {query} »");
        assert!(!visible(&out.shapes, "Saisir du texte"), "« {query} »");
    }
    // Les noms anglais fonctionnent aussi dans l'interface française.
    set_search(&h, "Wait for a window");
    let out = h.warm(size());
    assert!(visible(&out.shapes, "Attendre une fenêtre"));
    // Et le nom français dans l'interface anglaise.
    crate::i18n::set_language("en");
    set_search(&h, "ATTENDRE UNE FENETRE");
    let out = h.warm(size());
    assert!(visible(&out.shapes, "Wait for a window"));
    crate::i18n::set_language("fr");
}

#[test]
fn search_without_result_says_no_block_found_in_both_languages() {
    let mut h = scene(vec![delay(100)], vec![]);
    set_search(&h, "zzzxqj");
    assert!(visible(&h.warm(size()).shapes, NO_RESULT));
    crate::i18n::set_language("en");
    assert!(visible(&h.warm(size()).shapes, "No blocks found. Try another keyword."));
    crate::i18n::set_language("fr");
}

#[test]
fn clear_button_empties_the_search_and_restores_the_categories() {
    let mut h = scene(vec![delay(100)], vec![]);
    let out = type_in_search(&mut h, "zzzxqj");
    assert!(visible(&out.shapes, NO_RESULT));
    // Le × est dessiné à droite du champ : on clique à droite du texte saisi, dans le cadre.
    let field = at(&out.shapes, "zzzxqj");
    let panel = egui::containers::panel::PanelState::load(&h.ctx, egui::Id::new("workshop_library")).expect("bibliothèque visible").rect;
    let cross = egui::pos2(panel.right() - 22.0, field.y);
    h.click(size(), cross);
    let out = h.warm(size());
    assert_eq!(h.ctx.data(|d| d.get_temp::<String>(egui::Id::new("workshop_search"))).unwrap_or_default(), "");
    assert!(!visible(&out.shapes, NO_RESULT));
    assert!(visible(&out.shapes, "Clavier"));
}

#[test]
fn pinning_a_favorite_puts_it_first_and_survives_save_and_load() {
    let mut h = scene(vec![delay(100)], vec![]);
    set_search(&h, "Attendre");
    let out = h.warm(size());
    assert!(!visible(&out.shapes, "Favoris"));
    let row = rect_of(&h, "workshop_entry_rect", "cat:bloc:5");
    let star = rect_of(&h, "workshop_star_rect", "cat:bloc:5");
    h.frame(size(), vec![Input::PointerMoved(row.center())]);
    h.click(size(), star.center());
    assert_eq!(h.app.preferences.workshop_favorites, vec!["bloc:5".to_owned()]);
    set_search(&h, "");
    let out = h.warm(size());
    let (favorites, first_category) = (at(&out.shapes, "Favoris"), at(&out.shapes, "Clavier"));
    assert!(favorites.y < first_category.y, "la section Favoris est en tête : {favorites:?} / {first_category:?}");
    assert!(visible(&out.shapes, "Attendre"), "le favori est listé");
    // Persistance : vrai chemin de sauvegarde de l'application, puis rechargement.
    let saved = h.app.path.with_file_name("preferences.json");
    let loaded = crate::catalog::Preferences::load(&saved).expect("préférences relues");
    assert_eq!(loaded.workshop_favorites, vec!["bloc:5".to_owned()]);
    // Désépingler.
    let star = rect_of(&h, "workshop_star_rect", "fav:bloc:5");
    h.click(size(), star.center());
    assert!(h.app.preferences.workshop_favorites.is_empty());
    assert!(!visible(&h.warm(size()).shapes, "Favoris"));
    assert!(crate::catalog::Preferences::load(&saved).unwrap().workshop_favorites.is_empty());
}

#[test]
fn old_preferences_files_without_favorites_still_load() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("preferences.json");
    std::fs::write(&path, r#"{"dark": false}"#).unwrap();
    let loaded = crate::catalog::Preferences::load(&path).expect("ancien fichier lisible");
    assert!(loaded.workshop_favorites.is_empty());
    assert!(!loaded.dark);
}

#[test]
fn double_click_adds_the_block_at_the_end_with_a_single_block() {
    let mut h = scene(vec![delay(1), delay(2)], vec![]);
    h.warm(size());
    h.ctx.data_mut(|d| d.insert_temp(egui::Id::new("workshop_path"), vec![0usize]));
    set_search(&h, "Attendre");
    h.warm(size());
    let row = rect_of(&h, "workshop_entry_rect", "cat:bloc:5");
    let target = egui::pos2(row.left() + 70.0, row.center().y);
    h.click(size(), target);
    assert_eq!(steps_of(&h), vec![delay(1), delay(500), delay(2)], "un clic simple ajoute après la sélection");
    h.click(size(), target);
    assert_eq!(steps_of(&h), vec![delay(1), delay(2), delay(500)], "le double-clic place le bloc à la fin, sans doublon");
}

#[test]
fn plugin_actions_are_grouped_by_plugin_and_an_action_can_be_dragged_into_the_sequence() {
    let mut h = scene(vec![delay(1), delay(2)], vec![plugin("Alpha", "test.alpha", 2), plugin("Bêta", "test.beta", 2)]);
    let out = h.warm(tall());
    h.click(tall(), at(&out.shapes, "Plugins"));
    let out = h.warm(tall());
    assert!(visible(&out.shapes, "Alpha") && visible(&out.shapes, "Bêta"), "un en-tête par plugin");
    assert!(!visible(&out.shapes, "Alpha action 1"), "groupes repliés par défaut : aucun plugin ne masque les suivants");
    h.click(tall(), at(&out.shapes, "Alpha"));
    let out = h.warm(tall());
    h.click(tall(), at(&out.shapes, "Bêta"));
    let out = h.warm(tall());
    let ys: Vec<f32> = ["Alpha", "Alpha action 1", "Alpha action 2", "Bêta", "Bêta action 1", "Bêta action 2"].iter().map(|t| at(&out.shapes, t).y).collect();
    assert!(ys.windows(2).all(|w| w[0] < w[1]), "actions regroupées sous leur plugin, dans l'ordre : {ys:?}");
    // Glisser directement l'action de Bêta vers la séquence (avant le bloc 2).
    let source = at(&out.shapes, "Bêta action 2");
    let target = at(&out.shapes, "2");
    h.frame(tall(), vec![Input::PointerMoved(source)]);
    h.frame(tall(), vec![press(source, true)]);
    h.frame(tall(), vec![Input::PointerMoved(target)]);
    assert_eq!(h.ctx.data(|d| d.get_temp::<(String, String)>(egui::Id::new("workshop_palette_plugin"))),
        Some(("test.beta".to_owned(), "test.beta.action2".to_owned())), "glisser de plugin enregistré");
    h.frame(tall(), vec![Input::PointerMoved(target)]);
    h.frame(tall(), vec![press(target, false)]);
    let steps = steps_of(&h);
    assert_eq!(steps.len(), 3, "{steps:?}");
    assert!(matches!(&steps[1], Step::PluginAction { plugin, action, .. } if plugin == "test.beta" && action == "test.beta.action2"), "{steps:?}");
    assert!(h.ctx.data(|d| d.get_temp::<(String, String)>(egui::Id::new("workshop_palette_plugin"))).is_none(), "glisser terminé");
}

#[test]
fn plugin_search_opens_groups_and_pins_a_plugin_action_as_favorite() {
    let mut h = scene(vec![delay(1)], vec![plugin("Alpha", "test.alpha", 2), plugin("Bêta", "test.beta", 3)]);
    set_search(&h, "bêta action 3");
    let out = h.warm(size());
    assert!(visible(&out.shapes, "Bêta action 3"), "la recherche déplie le groupe");
    assert!(!visible(&out.shapes, "Alpha action 1") && !visible(&out.shapes, "Bêta action 1"));
    let key = "plugin:test.beta|test.beta.action3";
    let star = rect_of(&h, "workshop_star_rect", &format!("plugin:{key}"));
    h.click(size(), star.center());
    assert_eq!(h.app.preferences.workshop_favorites, vec![key.to_owned()]);
    let out = h.warm(size());
    assert!(visible(&out.shapes, "Favoris"), "action épinglée dans Favoris");
    // Désinstallation : le favori d'un plugin absent n'est plus proposé (mais reste mémorisé).
    h.app.plugins.as_mut().unwrap().set_catalog(vec![]);
    set_search(&h, "");
    let out = h.warm(size());
    assert!(!visible(&out.shapes, "Favoris") && !visible(&out.shapes, "Plugins"));
    assert_eq!(h.app.preferences.workshop_favorites, vec![key.to_owned()]);
}

fn plugin_card_scene(label: &str) -> (Harness, String, String) {
    let counter = plugin("Atelier Témoin", "test.card", 1);
    let (id, action) = (counter.uuid.clone(), counter.actions[0].uuid.clone());
    let mut h = scene(vec![Step::PluginAction { plugin: id.clone(), action: action.clone(), settings: serde_json::json!({}), label: label.into() }], vec![counter]);
    h.ctx.data_mut(|d| d.insert_temp(egui::Id::new("workshop_palette_open"), false));
    (h, id, action)
}

#[test]
fn plugin_card_shows_action_and_plugin_names_and_settings_button_opens_the_action_settings() {
    let (mut h, id, action) = plugin_card_scene("Mon compteur");
    let out = h.warm(size());
    assert!(visible(&out.shapes, "Atelier Témoin action 1"), "titre = nom de l'action");
    assert!(visible(&out.shapes, "Atelier Témoin · Mon compteur"), "résumé = plugin + libellé personnalisé");
    assert!(!visible(&out.shapes, "Plugin manquant"));
    assert!(h.app.block_inspector.is_none());
    h.click(size(), at(&out.shapes, "Réglages"));
    h.warm(size());
    let session = h.app.block_inspector.as_ref().expect("« Réglages » ouvre la fenêtre de réglages de l'action");
    assert_eq!(session.path, vec![0]);
    assert_eq!((session.plugin.as_str(), session.action.as_str()), (id.as_str(), action.as_str()));
    // Le bouton ne déclenche ni glisser ni suppression.
    assert_eq!(steps_of(&h).len(), 1);
}

#[test]
fn uninstalled_plugin_keeps_its_block_and_shows_a_missing_badge() {
    for lang in ["fr", "en"] {
        let (mut h, id, action) = plugin_card_scene("");
        crate::i18n::set_language(lang);
        let badge = crate::i18n::tr("Plugin manquant");
        let before = steps_of(&h);
        assert!(!visible(&h.warm(size()).shapes, badge), "plugin installé : pas de badge");
        h.app.plugins.as_mut().unwrap().set_catalog(vec![]);
        let out = h.warm(size());
        assert!(visible(&out.shapes, badge), "badge « {badge} »");
        assert!(!visible(&out.shapes, crate::i18n::tr("Réglages")), "pas de bouton de réglages sans plugin");
        assert_eq!(steps_of(&h), before, "étapes intactes");
        assert!(matches!(&steps_of(&h)[0], Step::PluginAction { plugin, action: a, .. } if *plugin == id && *a == action));
        // Réinstallé : le badge disparaît.
        h.app.plugins.as_mut().unwrap().set_catalog(vec![plugin("Atelier Témoin", "test.card", 1)]);
        assert!(!visible(&h.warm(size()).shapes, badge));
    }
    crate::i18n::set_language("fr");
}

#[test]
fn plugin_inventory_is_cached_between_frames_and_replaced_only_by_set_catalog() {
    let mut h = scene(vec![delay(1)], vec![plugin("Alpha", "test.alpha", 2)]);
    let first = h.app.plugins.as_ref().unwrap().inventory();
    h.warm(size());
    h.warm(size());
    let second = h.app.plugins.as_ref().unwrap().inventory();
    assert!(std::sync::Arc::ptr_eq(&first, &second), "même inventaire d'une image à l'autre (aucun rescan)");
    assert_eq!(second.len(), 1);
    h.app.plugins.as_mut().unwrap().set_catalog(vec![plugin("Alpha", "test.alpha", 2), plugin("Bêta", "test.beta", 1)]);
    let third = h.app.plugins.as_ref().unwrap().inventory();
    assert!(!std::sync::Arc::ptr_eq(&first, &third), "installation/mise à jour/désinstallation invalident le cache");
    assert_eq!(third.len(), 2);
    h.warm(size());
    assert!(std::sync::Arc::ptr_eq(&third, &h.app.plugins.as_ref().unwrap().inventory()));
}

#[test]
fn capture_library_favorites_plugins_and_search() {
    let Some(dir) = std::env::var_os("NOVYDECK_UI_OUTPUT") else { return };
    let dir = PathBuf::from(dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (lang, dark) in [("fr", true), ("en", true), ("fr", false)] {
        let logical = vec2(1280.0, 800.0) / 1.25;
        let counter = plugin("Compteur", "test.counter", 3);
        let clock = plugin("Horloge", "test.clock", 2);
        let card = Step::PluginAction { plugin: counter.uuid.clone(), action: counter.actions[1].uuid.clone(), settings: serde_json::json!({}), label: String::new() };
        let missing = Step::PluginAction { plugin: "test.removed".into(), action: "test.removed.run".into(), settings: serde_json::json!({}), label: "Ancien plugin".into() };
        let mut h = scene(vec![delay(500), card, missing], vec![counter.clone(), clock]);
        h.app.preferences.dark = dark;
        h.ctx.set_pixels_per_point(1.25);
        crate::i18n::set_language(lang);
        h.app.preferences.workshop_favorites = vec!["bloc:5".into(), format!("plugin:{}|{}", counter.uuid, counter.actions[0].uuid)];
        set_search(&h, "plugin");
        h.warm(logical);
        let output = h.warm(logical);
        assert!(visible(&output.shapes, crate::i18n::tr("Favoris")) && visible(&output.shapes, crate::i18n::tr("Plugins")));
        let suffix = if dark { lang.to_owned() } else { format!("{lang}-light") };
        h.png(logical, output, &dir.join(format!("workshop-library-{suffix}.png")));
    }
    crate::i18n::set_language("fr");
}
