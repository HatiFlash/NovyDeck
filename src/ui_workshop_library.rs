//! Bibliothèque de blocs de l'Atelier : champ de recherche, favoris, entrées avec pastille de catégorie,
//! glisser direct vers la séquence et plugins installés regroupés par plugin.
use super::*;

const ROW_HEIGHT: f32 = 30.0;
const STAR_ZONE: f32 = 26.0;

/// Résultat d'une image de la bibliothèque (appliqué par l'Atelier après le rendu).
#[derive(Default)]
pub(super) struct LibraryOut {
    pub add: Option<usize>,
    pub add_plugin: Option<Step>,
    /// Second clic d'un double-clic : le bloc ajouté par le premier clic part en fin de séquence.
    pub to_end: bool,
    /// Clé du favori à épingler ou à retirer.
    pub favorite: Option<String>,
}

/// Minuscules sans accents : « Fenêtre » et « FENETRE » se trouvent mutuellement.
pub(super) fn fold(text: &str) -> String {
    text.chars().flat_map(char::to_lowercase).map(|c| match c {
        'à' | 'â' | 'ä' | 'á' | 'ã' | 'å' => 'a',
        'é' | 'è' | 'ê' | 'ë' => 'e',
        'î' | 'ï' | 'í' | 'ì' => 'i',
        'ô' | 'ö' | 'ó' | 'ò' | 'õ' => 'o',
        'ù' | 'û' | 'ü' | 'ú' => 'u',
        'ç' => 'c',
        'ñ' => 'n',
        'ÿ' | 'ý' => 'y',
        '’' => '\'',
        other => other,
    }).collect()
}

fn tokens(search: &str) -> Vec<String> { fold(search).split_whitespace().map(str::to_owned).collect() }

fn all_match(haystack: &str, needles: &[String]) -> bool { needles.iter().all(|needle| haystack.contains(needle.as_str())) }

fn block_aliases(kind: usize) -> &'static str {
    match kind {
        19..=25 => "obs studio record stream scene source visibility virtual camera replay mute",
        0 => "hotkey super hotkeys discord browser back forward refresh quick control close window",
        1 => "text plain text",
        2 => "open openapps quick tool calculator mail music",
        3 => "website home page",
        4 => "multimedia systemvolume mute quick control",
        5 | 6 | 40 => "attente delai wait delay sleep pause",
        7..=11 => "pages folders",
        15 => "multi action",
        _ => "",
    }
}

/// Un bloc correspond si chaque mot cherché figure dans ses noms (français, anglais, langue active),
/// sa catégorie ou ses alias, sans tenir compte des accents ni de la casse.
pub(super) fn matches_palette(kind: usize, category: &str, search: &str) -> bool {
    let needles = tokens(search);
    if needles.is_empty() { return true; }
    let mut text = String::new();
    for form in crate::i18n::forms(block_name(kind)).into_iter().chain(crate::i18n::forms(category_name(category))) {
        text.push_str(&form);
        text.push(' ');
    }
    text.push_str(block_aliases(kind));
    all_match(&fold(&text), &needles)
}

fn plugin_matches(plugin: &crate::plugins::Plugin, action: &crate::plugins::Action, needles: &[String]) -> bool {
    needles.is_empty() || all_match(&fold(&format!("{} {} {} plugin", plugin.display_name(), action.name, action.uuid)), needles)
}

fn is_eligible(plugin: &crate::plugins::Plugin) -> bool {
    plugin.prerequisite.is_none()
        && matches!(plugin.runtime, crate::plugins::Runtime::Exe(_) | crate::plugins::Runtime::Node(_) | crate::plugins::Runtime::Html(_))
}

pub(super) fn plugin_key(plugin: &str, action: &str) -> String { format!("plugin:{plugin}|{action}") }

fn paint_magnifier(painter: &egui::Painter, center: egui::Pos2, color: Color32) {
    painter.circle_stroke(center - egui::vec2(1.5, 1.5), 5.0, egui::Stroke::new(1.6_f32, color));
    painter.line_segment([center + egui::vec2(2.0, 2.0), center + egui::vec2(6.5, 6.5)], egui::Stroke::new(1.8_f32, color));
}

fn paint_cross(painter: &egui::Painter, center: egui::Pos2, color: Color32) {
    let stroke = egui::Stroke::new(1.6_f32, color);
    painter.line_segment([center + egui::vec2(-3.5, -3.5), center + egui::vec2(3.5, 3.5)], stroke);
    painter.line_segment([center + egui::vec2(-3.5, 3.5), center + egui::vec2(3.5, -3.5)], stroke);
}

/// Étoile dessinée : pleine = favori, contour = épinglable (formes convexes uniquement).
fn paint_star(painter: &egui::Painter, center: egui::Pos2, filled: bool, color: Color32) {
    let points: Vec<egui::Pos2> = (0..10).map(|i| {
        let angle = -std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::PI / 5.0;
        let radius = if i % 2 == 0 { 7.5 } else { 3.2 };
        center + egui::vec2(angle.cos(), angle.sin()) * radius
    }).collect();
    if filled {
        let inner: Vec<egui::Pos2> = points.iter().skip(1).step_by(2).copied().collect();
        painter.add(egui::Shape::convex_polygon(inner, color, egui::Stroke::NONE));
        for i in (0..10).step_by(2) {
            painter.add(egui::Shape::convex_polygon(vec![points[(i + 9) % 10], points[i], points[(i + 1) % 10]], color, egui::Stroke::NONE));
        }
    } else {
        painter.add(egui::Shape::closed_line(points, egui::Stroke::new(1.3_f32, color)));
    }
}

fn row_galley(ui: &egui::Ui, text: &str, font: egui::FontId, color: Color32, width: f32) -> std::sync::Arc<egui::Galley> {
    let mut job = egui::text::LayoutJob::simple(text.to_owned(), font, color, width.max(20.0));
    job.wrap.max_rows = 1;
    job.wrap.break_anywhere = true;
    ui.painter().layout_job(job)
}

fn paint_badge(painter: &egui::Painter, rect: egui::Rect, style: &BlockStyle, texture: Option<egui::TextureId>) {
    painter.rect_filled(rect, 8.0, badge_fill(style.color));
    match texture {
        Some(id) => {
            let icon = egui::Rect::from_center_size(rect.center(), egui::vec2(BADGE_ICON, BADGE_ICON));
            painter.image(id, icon, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), Color32::WHITE);
        }
        None => iconset::paint(painter, rect.center(), BADGE_ICON, style.ico, style.family, 1.0, 0),
    }
}

/// Champ de recherche : loupe dessinée, effacement ×, bordure d'accent au focus.
pub(super) fn search_field(ui: &mut egui::Ui, search: &mut String) -> egui::Response {
    let p = theme::palette(ui.ctx());
    let edit_id = ui.id().with("workshop_search_edit");
    let focused = ui.memory(|m| m.has_focus(edit_id));
    let mut cleared = false;
    let framed = egui::Frame::new().fill(p.sunken)
        .stroke(egui::Stroke::new(if focused { 1.5_f32 } else { 1.0_f32 }, if focused { accent() } else { p.line }))
        .corner_radius(10).inner_margin(egui::Margin::symmetric(8, 4))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                ui.spacing_mut().interact_size.y = 22.0;
                let (icon, _) = ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
                paint_magnifier(ui.painter(), icon.center(), if focused { accent() } else { p.muted });
                let reserve = if search.is_empty() { 0.0 } else { 24.0 };
                let width = (ui.available_width() - reserve).max(40.0);
                let edit = ui.add(egui::TextEdit::singleline(&mut *search).id(edit_id).frame(false).desired_width(width)
                    .hint_text(crate::i18n::tr("Rechercher un bloc")));
                if !search.is_empty() {
                    let (rect, clear) = ui.allocate_exact_size(egui::vec2(18.0, 18.0), egui::Sense::click());
                    paint_cross(ui.painter(), rect.center(), if clear.hovered() { p.ink } else { p.muted });
                    if clear.on_hover_text(crate::i18n::tr("Effacer la recherche")).clicked() { cleared = true; }
                }
                edit
            }).inner
        });
    let mut edit = framed.inner;
    if cleared {
        search.clear();
        edit.mark_changed();
        edit.request_focus();
    }
    edit
}

struct Row { response: egui::Response, star: bool }

/// Une entrée : pastille icône/couleur de catégorie, nom, étoile au survol. Toute la ligne (sauf l'étoile)
/// se clique et se glisse ; l'infobulle n'est pas sélectionnable pour ne pas capter l'appui.
fn library_row(ui: &mut egui::Ui, salt: &str, step: &Step, texture: Option<egui::TextureId>, title: &str, help: &str,
    favorite: bool, cursor: bool) -> Row {
    let p = theme::palette(ui.ctx());
    let width = ui.available_width().max(80.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, ROW_HEIGHT), egui::Sense::hover());
    let main_rect = egui::Rect::from_min_max(rect.min, egui::pos2(rect.max.x - STAR_ZONE, rect.max.y));
    let star_rect = egui::Rect::from_min_max(egui::pos2(rect.max.x - STAR_ZONE, rect.min.y), rect.max);
    let response = ui.interact(main_rect, ui.id().with(("library_row", salt)), egui::Sense::click_and_drag());
    let star = ui.interact(star_rect, ui.id().with(("library_star", salt)), egui::Sense::click());
    #[cfg(test)]
    ui.ctx().data_mut(|d| {
        d.insert_temp(egui::Id::new(("workshop_entry_rect", salt.to_owned())), rect);
        d.insert_temp(egui::Id::new(("workshop_star_rect", salt.to_owned())), star_rect);
    });
    let hovered = response.hovered() || star.hovered();
    let painter = ui.painter().clone();
    if cursor || hovered {
        painter.rect_filled(rect, 8.0, if cursor { accent().gamma_multiply(0.18) } else { p.hover });
    }
    let badge = egui::Rect::from_min_size(egui::pos2(rect.min.x + 4.0, rect.center().y - 12.0), egui::vec2(24.0, 24.0));
    paint_badge(&painter, badge, &block_style(ui.ctx(), step), texture);
    let text_width = rect.width() - 4.0 - 24.0 - 8.0 - STAR_ZONE;
    let galley = row_galley(ui, title, egui::TextStyle::Body.resolve(ui.style()), p.ink, text_width);
    painter.galley(egui::pos2(badge.max.x + 8.0, rect.center().y - galley.size().y / 2.0), galley, p.ink);
    if favorite || hovered {
        paint_star(&painter, star_rect.center(), favorite, if favorite || star.hovered() { p.warning } else { p.muted });
    }
    let star = star.on_hover_text(if favorite { crate::i18n::tr("Retirer des favoris") } else { crate::i18n::tr("Épingler en favori") });
    let response = response.on_hover_ui(|ui| {
        ui.style_mut().interaction.selectable_labels = false;
        ui.set_max_width(260.0);
        ui.label(RichText::new(title).strong());
        if !help.is_empty() { ui.label(help); }
    }).on_hover_cursor(egui::CursorIcon::Grab);
    Row { response, star: star.clicked() }
}

fn block_row(ui: &mut egui::Ui, out: &mut LibraryOut, section: &str, kind: usize, favorites: &[String], cursor: bool) {
    let key = format!("bloc:{kind}");
    let row = library_row(ui, &format!("{section}:{key}"), &crate::automation::step(kind), None, block_name(kind), block_help(kind),
        favorites.contains(&key), cursor);
    if row.star { out.favorite = Some(key); }
    if row.response.double_clicked() { out.to_end = true; }
    else if row.response.clicked() { out.add = Some(kind); }
    if row.response.drag_started() {
        ui.ctx().data_mut(|d| {
            d.remove::<(String, String)>(egui::Id::new("workshop_palette_plugin"));
            d.insert_temp(egui::Id::new("workshop_palette_drag"), kind);
        });
    }
}

fn plugin_texture(ctx: &egui::Context, path: Option<&std::path::PathBuf>) -> Option<egui::TextureId> {
    path.and_then(|path| super::super::icon_ui::texture(ctx, &path.to_string_lossy())).map(|texture| texture.id())
}

fn plugin_row(ui: &mut egui::Ui, out: &mut LibraryOut, section: &str, plugin: &crate::plugins::Plugin, action: &crate::plugins::Action,
    favorites: &[String]) {
    let key = plugin_key(&plugin.uuid, &action.uuid);
    let title = if action.name.is_empty() { action.uuid.as_str() } else { action.name.as_str() };
    let texture = plugin_texture(ui.ctx(), action.icon.as_ref().or(plugin.icon.as_ref()));
    let row = library_row(ui, &format!("{section}:{key}"), &plugin_step(plugin, action), texture, title, plugin.display_name(),
        favorites.contains(&key), false);
    if row.star { out.favorite = Some(key); }
    if row.response.double_clicked() { out.to_end = true; }
    else if row.response.clicked() { out.add_plugin = Some(plugin_step(plugin, action)); }
    if row.response.drag_started() {
        ui.ctx().data_mut(|d| {
            d.remove::<usize>(egui::Id::new("workshop_palette_drag"));
            d.insert_temp(egui::Id::new("workshop_palette_plugin"), (plugin.uuid.clone(), action.uuid.clone()));
        });
    }
}

/// En-tête repliable d'un plugin : chevron, icône du manifeste (sinon pastille « plugin »), nom.
fn plugin_group(ui: &mut egui::Ui, out: &mut LibraryOut, plugin: &crate::plugins::Plugin, actions: &[&crate::plugins::Action],
    searching: bool, favorites: &[String]) {
    let p = theme::palette(ui.ctx());
    let open_id = egui::Id::new(("workshop_library_plugin_open", plugin.uuid.clone(), searching));
    let mut open = ui.ctx().data(|d| d.get_temp::<bool>(open_id)).unwrap_or(searching);
    let width = ui.available_width().max(80.0);
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, ROW_HEIGHT), egui::Sense::click());
    if response.clicked() {
        open = !open;
        ui.ctx().data_mut(|d| d.insert_temp(open_id, open));
    }
    #[cfg(test)]
    ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new(("workshop_plugin_group_rect", plugin.uuid.clone())), rect));
    let painter = ui.painter().clone();
    if response.hovered() { painter.rect_filled(rect, 8.0, p.hover); }
    let c = egui::pos2(rect.min.x + 10.0, rect.center().y);
    let points = if open { vec![c + egui::vec2(-4.0, -2.0), c + egui::vec2(0.0, 2.0), c + egui::vec2(4.0, -2.0)] }
        else { vec![c + egui::vec2(-2.0, -4.0), c + egui::vec2(2.0, 0.0), c + egui::vec2(-2.0, 4.0)] };
    painter.add(egui::Shape::line(points, egui::Stroke::new(1.8_f32, p.muted)));
    let group_step = Step::PluginAction { plugin: plugin.uuid.clone(), action: String::new(), settings: serde_json::json!({}), label: String::new() };
    let badge = egui::Rect::from_min_size(egui::pos2(rect.min.x + 22.0, rect.center().y - 12.0), egui::vec2(24.0, 24.0));
    paint_badge(&painter, badge, &block_style(ui.ctx(), &group_step), plugin_texture(ui.ctx(), plugin.icon.as_ref()));
    let galley = row_galley(ui, plugin.display_name(), theme::strong(13.0), p.ink, rect.max.x - badge.max.x - 16.0);
    painter.galley(egui::pos2(badge.max.x + 8.0, rect.center().y - galley.size().y / 2.0), galley, p.ink);
    if open {
        ui.indent(("workshop_library_plugin", plugin.uuid.clone()), |ui| {
            for action in actions { plugin_row(ui, out, "plugin", plugin, action, favorites); }
        });
    }
}

enum Favorite<'a> { Block(usize), Plugin(&'a crate::plugins::Plugin, &'a crate::plugins::Action) }

/// Affiche toute la bibliothèque (favoris, catégories, plugins). `keyboard` = entrée surlignée au clavier.
pub(super) fn show(ui: &mut egui::Ui, search: &str, advanced: bool, keyboard: Option<usize>, plugins: &[crate::plugins::Plugin],
    favorites: &[String]) -> LibraryOut {
    let mut out = LibraryOut::default();
    let needles = tokens(search);
    let searching = !needles.is_empty();
    let mut found = false;

    let pinned: Vec<Favorite> = favorites.iter().filter_map(|key| {
        if let Some(number) = key.strip_prefix("bloc:") {
            let kind: usize = number.parse().ok()?;
            if kind >= crate::automation::NAMES.len() || (!advanced && kind == 18) { return None; }
            let category = BLOCK_CATEGORIES.iter().find(|(_, kinds)| kinds.contains(&kind)).map_or("", |(name, _)| *name);
            matches_palette(kind, category, search).then_some(Favorite::Block(kind))
        } else if let Some(rest) = key.strip_prefix("plugin:") {
            let (uuid, action) = rest.split_once('|')?;
            let plugin = plugins.iter().find(|p| p.uuid == uuid && is_eligible(p))?;
            let action = plugin.actions.iter().find(|a| a.uuid == action)?;
            plugin_matches(plugin, action, &needles).then_some(Favorite::Plugin(plugin, action))
        } else { None }
    }).collect();
    if !pinned.is_empty() {
        found = true;
        egui::CollapsingHeader::new(crate::i18n::tr("Favoris")).id_salt(("library_favorites", searching)).default_open(true).show(ui, |ui| {
            for favorite in &pinned {
                match favorite {
                    Favorite::Block(kind) => block_row(ui, &mut out, "fav", *kind, favorites, false),
                    Favorite::Plugin(plugin, action) => plugin_row(ui, &mut out, "fav", plugin, action, favorites),
                }
            }
        });
    }

    let mut index = 0;
    for &(category, kinds) in BLOCK_CATEGORIES {
        let matches: Vec<usize> = kinds.iter().copied()
            .filter(|&kind| (advanced || kind != 18) && matches_palette(kind, category, search)).collect();
        if matches.is_empty() { continue; }
        found = true;
        let base = index;
        index += matches.len();
        egui::CollapsingHeader::new(category_name(category)).id_salt(("library_category", category, searching))
            .default_open(searching || category == "Clavier").show(ui, |ui| {
                for (n, kind) in matches.into_iter().enumerate() {
                    block_row(ui, &mut out, "cat", kind, favorites, keyboard == Some(base + n));
                }
            });
    }

    let groups: Vec<(&crate::plugins::Plugin, Vec<&crate::plugins::Action>)> = plugins.iter().filter(|p| is_eligible(p)).filter_map(|plugin| {
        let mut actions: Vec<&crate::plugins::Action> = plugin.actions.iter().collect();
        actions.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()).then_with(|| a.uuid.cmp(&b.uuid)));
        actions.retain(|action| plugin_matches(plugin, action, &needles));
        (!actions.is_empty()).then_some((plugin, actions))
    }).collect();
    if !groups.is_empty() {
        found = true;
        egui::CollapsingHeader::new(crate::i18n::tr("Plugins")).id_salt(("library_plugins", searching)).default_open(searching).show(ui, |ui| {
            for (plugin, actions) in &groups { plugin_group(ui, &mut out, plugin, actions, searching, favorites); }
        });
    }
    if !found { ui.label(crate::i18n::tr("Aucun bloc trouvé. Essaie un autre mot-clé.")); }
    out
}
