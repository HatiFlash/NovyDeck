//! Captures d'audit isolées : aucune action réelle, aucun accès réseau ni matériel.
use super::*;
use std::path::Path;

fn audit_macro(h: &mut Harness) {
    h.app.selected = 0;
    h.app.config.keys_mut()[0].label = "Démo · scénario complet".into();
    h.app.config.keys_mut()[0].steps = vec![
        Step::Launch { program: "brave.exe".into(), args: vec![], focus_if_running: true },
        Step::Delay { ms: 500 },
        Step::IfCondition {
            condition: crate::conditions::Condition::AppRunning { exe: "brave.exe".into() },
            then_steps: vec![Step::Hotkey { keys: "CTRL+T".into() }],
            else_steps: vec![Step::Launch { program: "brave.exe".into(), args: vec![], focus_if_running: false }],
        },
        Step::Repeat { count: 3, steps: vec![Step::Delay { ms: 250 }] },
        Step::Hotkey { keys: "CTRL+L".into() },
        Step::Delay { ms: 1000 },
    ];
    h.app.show_workflow = true;
}

fn capture(out: &Path, screen: &str, width: u32, height: u32, dpi: f32, dark: bool, lang: &str) {
    let physical = vec2(width as f32, height as f32);
    let size = physical / dpi;
    let theme_name = if dark { "sombre" } else { "clair" };
    let name = format!("audit-{screen}-{width}x{height}-{theme_name}-{lang}.png");
    // Une capture défaillante ne doit pas empêcher les autres écrans d'être rendus.
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut h = demo_harness();
        crate::i18n::set_language(lang); // Harness::new réinitialise la langue.
        h.app.preferences.language = lang.into();
        h.app.preferences.dark = dark;
        theme::apply(&h.ctx, dark);
        h.ctx.set_pixels_per_point(dpi);
        match screen {
            "deck" => { h.app.selected = 8; }
            "inspecteur-action" | "inspecteur-apparence" | "inspecteur-avance" => {
                h.app.selected = 0;
                let tab = match screen { "inspecteur-apparence" => 1u8, "inspecteur-avance" => 2u8, _ => 0u8 };
                h.ctx.data_mut(|d| d.insert_persisted(egui::Id::new("inspector_tab"), tab));
            }
            "ajouter" => {
                h.app.selected = 0;
                h.app.config.keys_mut()[0].steps.clear();
                let initial = h.warm(size);
                let pos = text_position(&initial.shapes, crate::i18n::tr("Ajouter"))
                    .ok_or("bouton Ajouter absent")?;
                h.click(size, pos);
            }
            "atelier-simple" | "atelier-avance" => {
                audit_macro(&mut h);
                h.app.preferences.workshop_advanced = screen == "atelier-avance";
                h.warm(size); // stabilise la cible avant la sélection du bloc
                h.ctx.data_mut(|d| d.insert_temp(egui::Id::new("workshop_path"), vec![1_usize]));
            }
            "bibliotheque" => {
                audit_macro(&mut h);
                h.warm(size);
                h.ctx.data_mut(|d| d.insert_temp(egui::Id::new("workshop_palette_open"), true));
            }
            "assistant" => {
                audit_macro(&mut h);
                h.app.workshop_chat.open = true;
                h.app.workshop_chat_send("ouvre le navigateur brave s'il est pas ouvert et si il deja ouvert ouvre un nouvel onglet".into(), None);
                if h.app.workshop_chat.pending.is_none() { eprintln!("AUDIT {name}: demande non reconnue"); }
            }
            "magasin-plugins" => { super::plugin_market_ui::test_view(None); }
            "plugins-installes" => { h.app.catalog_tab = 3; }
            "reglages" => { h.app.settings = true; }
            "pages-dossiers" => { h.app.page_manager = true; }
            "touche-vide" => {
                let selected = h.app.config.keys().iter().position(|key| key.steps.is_empty() && key.long_steps.is_empty() && key.plugin.is_none()).unwrap_or(14);
                h.app.selected = selected;
                h.app.config.keys_mut()[selected] = Key::default();
            }
            _ => return Err("écran inconnu"),
        }
        // Le PNG précède toute vérification d'éléments fragiles.
        let frame = h.warm(size);
        h.png(size, frame, &out.join(&name));
        if screen == "magasin-plugins" { super::plugin_market_ui::test_close(); }
        Ok::<(), &str>(())
    }));
    match result {
        Ok(Ok(())) => eprintln!("AUDIT OK {name}"),
        Ok(Err(reason)) => eprintln!("AUDIT MISSING {name}: {reason}"),
        Err(_) => { super::plugin_market_ui::test_close(); eprintln!("AUDIT MISSING {name}: panic de rendu ou écriture"); }
    }
}

fn audit_output() -> Option<PathBuf> {
    let out = std::env::var_os("NOVYDECK_UI_OUTPUT").map(PathBuf::from)?;
    std::fs::create_dir_all(&out).expect("dossier de captures");
    Some(out)
}

#[test]
fn ui_audit_captures_a() {
    let Some(out) = audit_output() else { return; };
    let all = ["deck", "inspecteur-action", "inspecteur-apparence", "inspecteur-avance", "ajouter",
        "atelier-simple", "atelier-avance", "bibliotheque", "assistant", "magasin-plugins",
        "plugins-installes", "reglages", "pages-dossiers", "touche-vide"];
    for screen in all { capture(&out, screen, 1366, 768, 1.0, true, "fr"); }
    crate::i18n::set_language("fr");
}

#[test]
fn ui_audit_captures_b() {
    let Some(out) = audit_output() else { return; };
    let extended = ["deck", "inspecteur-action", "inspecteur-apparence", "inspecteur-avance", "atelier-simple", "atelier-avance"];
    for screen in extended {
        capture(&out, screen, 1920, 1080, 1.5, true, "fr");
        capture(&out, screen, 1366, 768, 1.0, false, "fr");
        capture(&out, screen, 1366, 768, 1.0, true, "en");
    }
    crate::i18n::set_language("fr");
}
