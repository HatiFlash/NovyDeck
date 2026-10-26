//! Offline, bounded presentation of update and release information. No network or installation here.
use eframe::egui::{self, RichText, Vec2};
use crate::{i18n::{tr, trf}, theme};
use super::App;

#[derive(Debug, PartialEq, Eq)]
pub(super) enum NoteBlock { Heading(String), Bullet { title: String, body: String }, Paragraph(String) }

fn plain(raw: &str) -> String {
    let mut out = String::new();
    let mut rest = raw;
    while let Some(at) = rest.find('[') {
        out.push_str(&rest[..at]);
        let tail = &rest[at + 1..];
        if let Some((name, after)) = tail.split_once("](") {
            if let Some(end) = after.find(')') {
                out.push_str(name);
                rest = &after[end + 1..];
                continue;
            }
        }
        out.push('[');
        rest = tail;
    }
    out.push_str(rest);
    out.replace("**", "").replace('`', "").trim().to_owned()
}

pub(super) fn parse_notes(markdown: &str, lang: &str) -> Vec<NoteBlock> {
    let wanted = if lang == "fr" { "## Français" } else { "## English" };
    let lines: Vec<_> = markdown.lines().collect();
    let first = lines.iter().position(|line| line.trim() == wanted).or_else(||
        lines.iter().position(|line| matches!(line.trim(), "## Français" | "## English")));
    let selected = first.map(|n| &lines[n + 1..]).unwrap_or(&lines[..]);
    let mut blocks = Vec::new();
    let mut used = 0;
    for raw in selected {
        let line = raw.trim();
        if line.starts_with("## ") { break; }
        if line.is_empty() || line.starts_with("English · [Français](") || line.starts_with("Français · [English](") { continue; }
        let text = line.trim_start_matches('#').trim();
        if used + text.chars().count() > 4000 || blocks.len() >= 40 { break; }
        used += text.chars().count();
        if line.starts_with('#') { blocks.push(NoteBlock::Heading(plain(text))); }
        else if let Some(bullet) = line.strip_prefix("- ").or_else(|| line.strip_prefix("* ")) {
            if let Some(tail) = bullet.strip_prefix("**") {
                if let Some((title, body)) = tail.split_once("**") {
                    blocks.push(NoteBlock::Bullet { title: plain(title), body: plain(body) });
                    continue;
                }
            }
            blocks.push(NoteBlock::Bullet { title: String::new(), body: plain(bullet) });
        } else { blocks.push(NoteBlock::Paragraph(plain(text))); }
    }
    blocks
}

fn notes(ui: &mut egui::Ui, markdown: &str, cards: bool) {
    let blocks = parse_notes(markdown, crate::i18n::current());
    if blocks.is_empty() { return; }
    let p = theme::palette(ui.ctx());
    theme::overline(ui, tr("Nouveautés"));
    let max_height = (ui.ctx().screen_rect().height() - 310.0).clamp(100.0, 305.0);
    egui::ScrollArea::vertical().id_salt(if cards { "whats_new_notes" } else { "update_notes" })
        .max_height(max_height).show(ui, |ui| {
            ui.set_max_width(ui.available_width());
            for block in blocks {
                match block {
                    NoteBlock::Heading(title) => { ui.add_space(6.0); ui.label(RichText::new(title).strong().size(16.0).color(p.ink)); }
                    NoteBlock::Bullet { title, body } => {
                        let content = |ui: &mut egui::Ui| {
                            ui.horizontal_top(|ui| {
                                ui.label(RichText::new("✦").color(p.accent));
                                ui.vertical(|ui| {
                                    ui.set_max_width((ui.available_width() - 24.0).max(100.0));
                                    if !title.is_empty() { ui.label(RichText::new(title).strong().color(p.ink)); }
                                    if !body.is_empty() { ui.add(egui::Label::new(body).wrap()); }
                                });
                            });
                        };
                        if cards { theme::card(p).show(ui, content); ui.add_space(6.0); }
                        else { content(ui); ui.add_space(5.0); }
                    }
                    NoteBlock::Paragraph(text) => { ui.add(egui::Label::new(text).wrap()); }
                }
            }
        });
}

fn modal<T>(ctx: &egui::Context, response: Option<egui::InnerResponse<T>>) {
    if let Some(shown) = response {
        let layer = shown.response.layer_id;
        ctx.move_to_top(layer);
        ctx.memory_mut(|memory| memory.set_modal_layer(layer));
    }
}

fn brand(ui: &mut egui::Ui, title: &str, subtitle: &str) {
    let p = theme::palette(ui.ctx());
    theme::card(p).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.add(egui::Image::new(&theme::logo_texture(ui.ctx())).fit_to_exact_size(Vec2::splat(44.0)));
            ui.vertical(|ui| {
                ui.label(RichText::new(title).size(20.0).strong().color(p.ink));
                ui.label(RichText::new(subtitle).color(p.text_secondary));
            });
        });
    });
}

pub(super) fn whats_new(ctx: &egui::Context, markdown: &str) -> bool {
    let mut open = true;
    let mut done = false;
    let shown = theme::fixed_window(trf("Quoi de neuf dans NovyDeck {version}", &[("version", &env!("CARGO_PKG_VERSION"))]))
        .id(egui::Id::new("novy_whats_new"))
        .open(&mut open).resizable(false).default_width(560.0)
        .show(ctx, |ui| {
            ui.set_max_width(ui.available_width().min(560.0));
            brand(ui, &format!("NovyDeck {}", env!("CARGO_PKG_VERSION")), tr("Les nouveautés de cette version"));
            ui.add_space(8.0);
            notes(ui, markdown, true);
            ui.separator();
            ui.horizontal_wrapped(|ui| {
                if ui.add(theme::primary(tr("C'est parti"))).clicked() { done = true; }
                if ui.add(theme::ghost_button(tr("Notes complètes sur GitHub"))).clicked() {
                    ctx.open_url(egui::OpenUrl::new_tab(format!("https://github.com/HatiFlash/NovyDeck/releases/tag/v{}", env!("CARGO_PKG_VERSION"))));
                }
            });
        });
    modal(ctx, shown);
    done || !open
}

#[derive(PartialEq, Eq)]
pub(super) enum OfferAction { None, Later, Install }

pub(super) fn offer(ctx: &egui::Context, latest: &crate::app_update::Latest, installed: bool) -> OfferAction {
    let mut open = true;
    let mut action = OfferAction::None;
    let shown = theme::fixed_window(tr("Mise à jour disponible"))
        .id(egui::Id::new("novy_update_offer"))
        .open(&mut open).resizable(false).default_width(570.0)
        .show(ctx, |ui| {
            ui.set_max_width(ui.available_width().min(570.0));
            brand(ui, &format!("NovyDeck {} → {}", env!("CARGO_PKG_VERSION"), latest.version.trim_start_matches('v')),
                &trf("Installateur · {size} Mo · SHA-256 vérifié avant installation", &[("size", &format!("{:.1}", latest.setup_size as f64 / 1_000_000.0))]));
            ui.add_space(8.0);
            notes(ui, &latest.notes, false);
            ui.label(RichText::new(tr("Profils, médias et plugins conservés. NovyDeck redémarrera après l'installation.")).small());
            if !installed { ui.label(tr("Version portable : téléchargez le ZIP depuis la page des nouveautés")); }
            ui.separator();
            ui.horizontal_wrapped(|ui| {
                if installed && ui.add(theme::primary(tr("Mettre à jour maintenant"))).clicked() { action = OfferAction::Install; }
                if ui.add(theme::ghost_button(tr("Plus tard"))).clicked() { action = OfferAction::Later; }
                if ui.add(theme::ghost_button(tr("Voir sur GitHub"))).clicked() {
                    ctx.open_url(egui::OpenUrl::new_tab(latest.page.clone()));
                }
            });
        });
    modal(ctx, shown);
    if !open { OfferAction::Later } else { action }
}

pub(super) fn status(ui: &mut egui::Ui, app: &App) {
    let p = theme::palette(ui.ctx());
    if let Some(phase) = app.update_phase {
        ui.add_space(5.0);
        ui.label(RichText::new(phase).strong());
        let verifying = phase == tr("Vérification…");
        ui.add(egui::ProgressBar::new(if verifying { 0.62 } else { 0.25 }).animate(true).desired_width(ui.available_width()));
        ui.label(RichText::new(tr("Téléchargement → Vérification → Installation → Relance")).small().color(p.text_secondary));
        if let Some(latest) = app.startup_offer.as_ref().or_else(|| app.update_confirm.as_ref()) {
            ui.label(trf("{size} octets annoncés par la version publiée", &[("size", &latest.setup_size)]));
        }
    }
    if let Some(result) = &app.update_result {
        match result {
            Err(error) => { theme::card(p).show(ui, |ui| { ui.colored_label(p.danger, error); }); }
            Ok(crate::app_update::Status::Current) => { ui.label(RichText::new(trf("À jour (version {version})", &[("version", &env!("CARGO_PKG_VERSION"))])).color(p.success)); }
            Ok(crate::app_update::Status::Ahead) => { ui.label(tr("Cette version est plus récente que la dernière publiée")); }
            Ok(crate::app_update::Status::New(_)) => {}
        }
    }
}

pub(super) fn confirm(ctx: &egui::Context, latest: &crate::app_update::Latest) -> Option<bool> {
    let mut open = true;
    let mut choice = None;
    let shown = theme::fixed_window(tr("Installer la mise à jour"))
        .id(egui::Id::new("novy_update_confirm"))
        .open(&mut open).resizable(false).default_width(530.0)
        .show(ctx, |ui| {
            brand(ui, &format!("NovyDeck {} → {}", env!("CARGO_PKG_VERSION"), latest.version.trim_start_matches('v')), tr("Confirmation requise avant téléchargement"));
            notes(ui, &latest.notes, false);
            ui.label(trf("NovyDeck va se fermer, installer la version {version} puis redémarrer. Vos profils, médias et plugins sont conservés.", &[("version", &latest.version)]));
            ui.horizontal_wrapped(|ui| {
                if ui.add(theme::primary(tr("Mettre à jour maintenant"))).clicked() { choice = Some(true); }
                if ui.add(theme::ghost_button(tr("Annuler"))).clicked() { choice = Some(false); }
            });
        });
    modal(ctx, shown);
    if open { choice } else { Some(false) }
}
