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
                    let title = plain(title).trim_end_matches(':').trim().to_owned();
                    let body = plain(body);
                    let mut chars = body.chars();
                    let body = chars.next().map(|c| c.to_uppercase().collect::<String>() + chars.as_str()).unwrap_or_default();
                    blocks.push(NoteBlock::Bullet { title, body });
                    continue;
                }
            }
            blocks.push(NoteBlock::Bullet { title: String::new(), body: plain(bullet) });
        } else { blocks.push(NoteBlock::Paragraph(plain(text))); }
    }
    blocks
}

pub(super) fn notes(ui: &mut egui::Ui, markdown: &str, cards: bool) {
    let blocks = parse_notes(markdown, crate::i18n::current());
    if blocks.is_empty() { return; }
    let p = theme::palette(ui.ctx());
    theme::overline(ui, tr("Nouveautés"));
    let max_height = (ui.ctx().screen_rect().height() - 440.0).clamp(100.0, 180.0);
    egui::ScrollArea::vertical().id_salt(if cards { "whats_new_notes" } else { "update_notes" })
        .max_height(max_height).show(ui, |ui| {
            ui.set_max_width(ui.available_width());
            for block in blocks {
                match block {
                    NoteBlock::Heading(title) => { ui.add_space(6.0); ui.label(RichText::new(title).strong().size(16.0).color(p.ink)); }
                    NoteBlock::Bullet { title, body } => {
                        let content = |ui: &mut egui::Ui| {
                            ui.horizontal_top(|ui| {
                                let (rect, _) = ui.allocate_exact_size(Vec2::splat(18.0), egui::Sense::hover());
                                ui.painter().circle_filled(rect.center(), 4.0, p.accent);
                                ui.vertical(|ui| {
                                    ui.set_max_width((ui.available_width() - 8.0).max(100.0));
                                    if !title.is_empty() { ui.label(RichText::new(title).strong().size(15.0).color(p.ink)); }
                                    if !body.is_empty() { ui.add(egui::Label::new(body).wrap()); }
                                });
                            });
                        };
                        let width = ui.available_width();
                        if cards { theme::card(p).show(ui, |ui| { ui.set_min_width((width - 24.0).max(100.0)); content(ui); }); ui.add_space(6.0); }
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

fn brand(ui: &mut egui::Ui, title: &str, subtitle: &str, sha: bool) {
    let p = theme::palette(ui.ctx());
    let width = ui.available_width();
    theme::card(p).show(ui, |ui| {
        ui.set_min_width((width - 24.0).max(100.0));
        ui.horizontal(|ui| {
            ui.add(egui::Image::new(&theme::logo_texture(ui.ctx())).fit_to_exact_size(Vec2::splat(44.0)));
            ui.vertical(|ui| {
                ui.label(RichText::new(title).size(20.0).strong().color(p.ink));
                ui.label(RichText::new(subtitle).color(p.text_secondary));
                if sha { ui.label(RichText::new(tr("SHA-256 vérifié avant installation")).small().strong().color(p.success)); }
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
            brand(ui, &format!("NovyDeck {}", env!("CARGO_PKG_VERSION")), tr("Les nouveautés de cette version"), false);
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
                &trf("Installateur · {size} Mo", &[("size", &format!("{:.1}", latest.setup_size as f64 / 1_000_000.0))]), true);
            ui.add_space(8.0);
            notes(ui, &latest.notes, true);
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
    if app.update_phase == Some(tr("Vérification en cours…")) { ui.label(tr("Vérification en cours…")); }
    if let Some(result) = &app.update_result {
        match result {
            Err(error) if !app.update_installing => { ui.colored_label(theme::palette(ui.ctx()).danger, error); }
            Ok(crate::app_update::Status::Current) => { ui.label(trf("À jour (version {version})", &[("version", &env!("CARGO_PKG_VERSION"))])); }
            Ok(crate::app_update::Status::Ahead) => { ui.label(tr("Cette version est plus récente que la dernière publiée")); }
            _ => {}
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum StepState { Done, Active, Failed, Waiting }

pub(super) fn step_state(index: usize, phase: Option<&str>, failed: bool) -> StepState {
    let current = if phase == Some(tr("Vérification…")) { 1 }
        else if phase == Some(tr("Installation…")) { 2 } else { 0 };
    if index < current { StepState::Done }
    else if index == current && failed { StepState::Failed }
    else if index == current { StepState::Active }
    else { StepState::Waiting }
}

pub(super) fn operation(ctx: &egui::Context, app: &App) -> bool {
    let mut retry = false;
    let error = app.update_result.as_ref().and_then(|result| result.as_ref().err());
    let shown = theme::fixed_window(tr("Installation de la mise à jour"))
        .id(egui::Id::new("novy_update_operation")).resizable(false).default_width(570.0)
        .show(ctx, |ui| {
            let p = theme::palette(ui.ctx());
            ui.set_min_width(ui.available_width().min(570.0));
            let title = app.update_target.as_ref().map_or_else(
                || tr("Mise à jour de NovyDeck").to_owned(),
                |latest| format!("NovyDeck {} → {}", env!("CARGO_PKG_VERSION"), latest.version.trim_start_matches('v')),
            );
            brand(ui, &title, tr("Mise à jour de NovyDeck"), false);
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                for (index, step) in [tr("Téléchargement"), tr("Vérification SHA-256"), tr("Installation"), tr("Relance")].iter().enumerate() {
                    let state = step_state(index, app.update_phase, error.is_some());
                    let color = match state {
                        StepState::Done => p.success,
                        StepState::Active => p.accent,
                        StepState::Failed => p.danger,
                        StepState::Waiting => p.text_secondary,
                    };
                    let (rect, _) = ui.allocate_exact_size(Vec2::splat(17.0), egui::Sense::hover());
                    let center = rect.center();
                    ui.painter().circle_filled(center, 7.0, color);
                    let stroke = egui::Stroke::new(1.7, p.raised);
                    match state {
                        StepState::Done => {
                            ui.painter().line_segment([center + Vec2::new(-3.5, 0.0), center + Vec2::new(-1.0, 2.5)], stroke);
                            ui.painter().line_segment([center + Vec2::new(-1.0, 2.5), center + Vec2::new(4.0, -3.0)], stroke);
                        }
                        StepState::Failed => {
                            ui.painter().line_segment([center + Vec2::new(-2.5, -2.5), center + Vec2::new(2.5, 2.5)], stroke);
                            ui.painter().line_segment([center + Vec2::new(2.5, -2.5), center + Vec2::new(-2.5, 2.5)], stroke);
                        }
                        _ => {}
                    }
                    ui.label(RichText::new(*step).color(color).strong());
                }
            });
            ui.add_space(10.0);
            if let Some(message) = error {
                let width = ui.available_width();
                theme::card(p).show(ui, |ui| {
                    ui.set_min_width((width - 24.0).max(100.0));
                    ui.horizontal_top(|ui| {
                        let (rect, _) = ui.allocate_exact_size(Vec2::splat(20.0), egui::Sense::hover());
                        ui.painter().circle_filled(rect.center(), 8.0, p.danger);
                        ui.add(egui::Label::new(RichText::new(message).color(p.danger)).wrap());
                    });
                });
                ui.horizontal(|ui| {
                    if ui.add(theme::primary(tr("Réessayer"))).clicked() { retry = true; }
                    if ui.add(theme::ghost_button(tr("Voir sur GitHub"))).clicked() {
                        ctx.open_url(egui::OpenUrl::new_tab("https://github.com/HatiFlash/NovyDeck/releases"));
                    }
                });
            } else {
                ui.label(RichText::new(app.update_phase.unwrap_or(tr("Installation…"))).strong());
                // The downloader only reports a phase, not received bytes. Never invent a percentage.
                let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 8.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, 4.0, p.sunken);
                let x = rect.left() + ((ui.input(|i| i.time) as f32 * 65.0) % (rect.width() - 48.0).max(1.0));
                ui.painter().rect_filled(egui::Rect::from_min_size(egui::pos2(x, rect.top()), Vec2::new(48.0, 8.0)), 4.0, p.accent);
                ui.label(RichText::new(tr("Progression indéterminée — octets reçus indisponibles")).small().color(p.text_secondary));
                ctx.request_repaint();
            }
        });
    modal(ctx, shown);
    retry
}

pub(super) fn confirm(ctx: &egui::Context, latest: &crate::app_update::Latest) -> Option<bool> {
    let mut open = true;
    let mut choice = None;
    let shown = theme::fixed_window(tr("Installer la mise à jour"))
        .id(egui::Id::new("novy_update_confirm"))
        .open(&mut open).resizable(false).default_width(530.0)
        .show(ctx, |ui| {
            brand(ui, &format!("NovyDeck {} → {}", env!("CARGO_PKG_VERSION"), latest.version.trim_start_matches('v')), tr("Confirmation requise avant téléchargement"), true);
            notes(ui, &latest.notes, true);
            ui.label(trf("NovyDeck va se fermer, installer la version {version} puis redémarrer. Vos profils, médias et plugins sont conservés.", &[("version", &latest.version)]));
            ui.horizontal_wrapped(|ui| {
                if ui.add(theme::primary(tr("Mettre à jour maintenant"))).clicked() { choice = Some(true); }
                if ui.add(theme::ghost_button(tr("Annuler"))).clicked() { choice = Some(false); }
            });
        });
    modal(ctx, shown);
    if open { choice } else { Some(false) }
}
