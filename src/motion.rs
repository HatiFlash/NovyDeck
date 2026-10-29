//! Petites animations d'interface : intro au démarrage, indicateur de chargement, fondus de
//! changement de page ou d'onglet.
//!
//! Règles : rien ne bloque la souris ni le clavier, rien ne déplace les éléments cliquables
//! (seule l'opacité varie), aucune image n'est redessinée en continu hors animation, et
//! l'option « Réduire les animations » coupe tout sauf l'indicateur de chargement.
use eframe::egui::{self, Color32, Vec2};
use std::hash::Hash;

/// Windows accessibility setting, read once at startup (headless tests keep animations enabled).
pub fn system_animations() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if cfg!(test) { return true; } // CI runners disable Windows animations; tests drive the preference.
    *ENABLED.get_or_init(|| {
        #[cfg(windows)]
        {
            let mut enabled: i32 = 1;
            // A failed query must not silently turn off the user's saved preference.
            let ok = unsafe { windows_sys::Win32::UI::WindowsAndMessaging::SystemParametersInfoW(
                windows_sys::Win32::UI::WindowsAndMessaging::SPI_GETCLIENTAREAANIMATION,
                0, &mut enabled as *mut i32 as *mut _, 0,
            ) };
            ok == 0 || enabled != 0
        }
        #[cfg(not(windows))]
        { true }
    })
}

/// Durée d'un fondu de page ou d'onglet.
pub const FADE: f64 = 0.24;
/// Durée totale de l'intro (logo, puis fondu vers l'interface).
pub const INTRO: f64 = 1.15;
const INTRO_HOLD: f64 = 0.6;

/// Décélération douce (ease-out cubique).
pub fn ease_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

fn now(ctx: &egui::Context) -> f64 {
    ctx.input(|i| i.time)
}

/// Avancement 0 → 1 depuis le dernier changement de `key` pour `id`.
/// Premier affichage : pas d'animation (1.0).
pub fn progress(ctx: &egui::Context, id: egui::Id, key: impl Hash, seconds: f64, enabled: bool) -> f32 {
    let key = egui::Id::new(key).value();
    let time = now(ctx);
    let (last, start) = ctx.data_mut(|d| *d.get_temp_mut_or_insert_with(id, || (key, f64::NEG_INFINITY)));
    let start = if last != key {
        ctx.data_mut(|d| d.insert_temp(id, (key, time)));
        time
    } else {
        start
    };
    if !enabled {
        return 1.0;
    }
    let t = ((time - start) / seconds) as f32;
    if t < 1.0 {
        ctx.request_repaint();
    }
    ease_out(t)
}

/// Fondu du reste du contenu de `ui` quand `key` change (page, onglet, profil…).
/// L'opacité part de `from` pour rester discrète ; les positions ne bougent pas.
pub fn fade(ui: &mut egui::Ui, id: &str, key: impl Hash, from: f32, enabled: bool) {
    let seconds = if id == "settings_appear" || id == "activity" { 0.18 } else { FADE };
    let t = progress(ui.ctx(), egui::Id::new(("motion_fade", id)), key, seconds, enabled);
    if t < 1.0 {
        ui.set_opacity(from + (1.0 - from) * t);
    }
}

/// Interpole une nouvelle couleur de boîtier sans toucher aux rectangles ni au choix sauvegardé.
pub fn deck_color(ctx: &egui::Context, color: Option<[u8; 3]>, enabled: bool) -> Option<[u8; 3]> {
    let id = egui::Id::new("deck_color_previous");
    let (previous, current) = ctx.data_mut(|d| {
        let pair = d.get_temp_mut_or_insert_with(id, || (color, color));
        if pair.1 != color { *pair = (pair.1, color); }
        *pair
    });
    let t = progress(ctx, id.with("fade"), color, 0.18, enabled);
    if t >= 1.0 { return current; }
    let old = previous.unwrap_or([30, 35, 42]);
    let new = current.unwrap_or([30, 35, 42]);
    Some(std::array::from_fn(|i| (old[i] as f32 + (new[i] as f32 - old[i] as f32) * t).round() as u8))
}

fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let m = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgb(m(a.r(), b.r()), m(a.g(), b.g()), m(a.b(), b.b()))
}

/// Peint l'indicateur NovyDeck : 4 touches (2 × 2) qui s'allument tour à tour, comme le logo.
pub fn paint_loader(painter: &egui::Painter, center: egui::Pos2, size: f32, accent: Color32, time: f64) {
    let gap = size * 0.12;
    let cell = (size - gap) / 2.0;
    // Ordre horaire : haut gauche, haut droite, bas droite, bas gauche.
    let cells = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];
    let phase = (time * 1.6).rem_euclid(1.0) as f32 * 4.0;
    let origin = center - Vec2::splat(size / 2.0);
    let dim = mix(accent, Color32::from_gray(128), 0.55);
    for (i, (x, y)) in cells.iter().enumerate() {
        // Distance (en tours) derrière la touche active : la lumière s'estompe en traînée.
        let behind = (phase - i as f32).rem_euclid(4.0);
        let glow = (1.0 - behind / 2.2).clamp(0.0, 1.0);
        let min = origin + Vec2::new(x * (cell + gap), y * (cell + gap));
        let grow = cell * 0.08 * glow;
        let rect = egui::Rect::from_min_size(min, Vec2::splat(cell)).expand(grow);
        let color = mix(dim, accent, glow).gamma_multiply(0.45 + 0.55 * glow);
        painter.rect_filled(rect, cell * 0.28, color);
    }
}

/// Indicateur de chargement (remplace `ui.spinner()`), animé tant qu'il est visible.
pub fn loader(ui: &mut egui::Ui, size: f32) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), egui::Sense::hover());
    if ui.is_rect_visible(rect) {
        paint_loader(ui.painter(), rect.center(), size, crate::theme::accent(), now(ui.ctx()));
        ui.ctx().request_repaint();
    }
    response
}

/// Indicateur suivi d'un texte, sur une ligne.
pub fn loading(ui: &mut egui::Ui, text: impl Into<egui::WidgetText>) {
    ui.horizontal(|ui| {
        loader(ui, 16.0);
        ui.label(text);
    });
}

/// Point d'état qui « respire » pendant une attente (connexion du périphérique…).
pub fn status_dot(painter: &egui::Painter, center: egui::Pos2, color: Color32, waiting: bool, enabled: bool) {
    if waiting && enabled {
        let t = (now(painter.ctx()) * 1.4).rem_euclid(1.0) as f32;
        painter.circle_filled(center, 4.0 + 6.0 * ease_out(t), color.gamma_multiply(0.45 * (1.0 - t)));
        painter.ctx().request_repaint();
    }
    painter.circle_filled(center, 4.0, color);
}

/// Intro du démarrage : logo qui apparaît en grandissant, nom, indicateur, puis fondu vers l'interface.
/// Peinte au premier plan sans capter la souris ; disparaît d'elle-même après `INTRO` secondes.
pub fn intro(ctx: &egui::Context, logo: &egui::TextureHandle, enabled: bool) -> bool {
    let id = egui::Id::new("motion_intro_start");
    let time = now(ctx);
    let start = ctx.data_mut(|d| *d.get_temp_mut_or_insert_with(id, || time));
    let elapsed = time - start;
    if !enabled || elapsed >= INTRO {
        return false;
    }
    let p = crate::theme::palette(ctx);
    let screen = ctx.screen_rect();
    let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("motion_intro")));
    let veil = if elapsed < INTRO_HOLD { 1.0 } else { 1.0 - ease_out(((elapsed - INTRO_HOLD) / (INTRO - INTRO_HOLD)) as f32) };
    painter.rect_filled(screen, 0.0, p.bg.gamma_multiply(veil));
    let appear = ease_out((elapsed / 0.45) as f32);
    let alpha = appear * veil;
    let center = screen.center() - Vec2::new(0.0, 30.0);
    let size = 112.0 * (0.82 + 0.18 * appear);
    painter.image(
        logo.id(),
        egui::Rect::from_center_size(center - Vec2::new(0.0, 10.0 * (1.0 - appear)), Vec2::splat(size)),
        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
        Color32::WHITE.gamma_multiply(alpha),
    );
    painter.text(
        center + Vec2::new(0.0, 80.0),
        egui::Align2::CENTER_CENTER,
        "NovyDeck",
        egui::FontId::proportional(26.0),
        p.ink.gamma_multiply(alpha),
    );
    paint_loader(&painter, center + Vec2::new(0.0, 124.0), 22.0, p.accent.gamma_multiply(alpha), time);
    ctx.request_repaint();
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(ctx: &egui::Context, time: f64, add: impl FnMut(&mut egui::Ui)) {
        let input = egui::RawInput { time: Some(time), ..Default::default() };
        let mut add = add;
        let _ = ctx.run(input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| add(ui));
        });
    }

    #[test]
    fn easing_is_bounded_and_monotonic() {
        assert_eq!(ease_out(-1.0), 0.0);
        assert_eq!(ease_out(2.0), 1.0);
        let mut last = 0.0;
        for i in 1..=20 {
            let v = ease_out(i as f32 / 20.0);
            assert!(v > last);
            last = v;
        }
    }

    #[test]
    fn fade_runs_only_after_a_change_and_ends_on_time() {
        let ctx = egui::Context::default();
        let mut seen = Vec::new();
        let mut key = 1;
        for (i, time) in [0.0, 0.1, 0.2, 0.3, 0.32, 0.4, 0.6].into_iter().enumerate() {
            if i == 3 {
                key = 2;
            }
            frame(&ctx, time, |ui| seen.push(progress(ui.ctx(), egui::Id::new("t"), key, FADE, true)));
        }
        assert_eq!(&seen[..3], &[1.0, 1.0, 1.0], "pas d'animation au premier affichage");
        assert_eq!(seen[3], 0.0, "le changement relance le fondu");
        assert!(seen[4] > 0.0 && seen[4] < 1.0);
        assert_eq!(seen[6], 1.0, "fini après {FADE} s");
        let ctx = egui::Context::default();
        frame(&ctx, 0.0, |ui| assert_eq!(progress(ui.ctx(), egui::Id::new("t"), 1, FADE, false), 1.0));
        frame(&ctx, 0.1, |ui| assert_eq!(progress(ui.ctx(), egui::Id::new("t"), 2, FADE, false), 1.0, "« Réduire les animations »"));
    }

    #[test]
    fn repaint_stops_after_transition_and_is_absent_when_disabled() {
        let ctx = egui::Context::default();
        let id = egui::Id::new("repaint_transition");
        let mut delays = Vec::new();
        let mut values = Vec::new();
        for (time, key, enabled) in [(0.0, 0, true), (0.1, 1, true), (0.2, 1, true), (0.35, 1, true), (0.5, 1, true), (0.6, 2, false)] {
            let output = ctx.run(egui::RawInput { time: Some(time), ..Default::default() }, |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| values.push(progress(ui.ctx(), id, key, 0.18, enabled)));
            });
            delays.push(output.viewport_output[&egui::ViewportId::ROOT].repaint_delay);
        }
        assert!(delays[1].is_zero(), "repaint pendant le fondu");
        assert_eq!(values[3], 1.0);
        assert!(!delays[4].is_zero(), "pas de boucle de rendu après la fin");
        assert_eq!(values[5], 1.0);
        assert!(!delays[5].is_zero(), "préférence désactivée : pas de repaint");
    }

    #[test]
    fn intro_shows_then_disappears_and_can_be_disabled() {
        let ctx = egui::Context::default();
        let logo = ctx.load_texture("logo", egui::ColorImage::new([4, 4], Color32::WHITE), Default::default());
        let mut shown = Vec::new();
        for time in [0.0, 0.3, 0.9, INTRO + 0.01, 3.0] {
            let input = egui::RawInput { time: Some(time), ..Default::default() };
            let _ = ctx.run(input, |ctx| shown.push(intro(ctx, &logo, true)));
        }
        assert_eq!(shown, [true, true, true, false, false]);
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| assert!(!intro(ctx, &logo, false)));
    }
}
