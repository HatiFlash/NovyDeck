use super::*;

fn sample() -> crate::app_update::Latest {
    crate::app_update::Latest {
        version: "v0.13.0".into(),
        page: "https://github.com/HatiFlash/NovyDeck/releases/tag/v0.13.0".into(),
        setup_url: "https://github.com/example/setup.exe".into(),
        sha_url: "https://github.com/example/setup.sha256".into(),
        setup_size: 32_456_789,
        notes: include_str!("../docs/RELEASE-0.13.0.md").into(),
    }
}

#[test]
fn update_before_captures_offline() {
    let Some(out) = std::env::var_os("NOVYDECK_UI_OUTPUT") else { return; };
    let out = PathBuf::from(out);
    let size = vec2(1366.0, 768.0);
    for state in ["disponible", "telechargement", "verification-sha", "erreur-reseau", "a-jour", "quoi-de-neuf"] {
        let mut h = Harness::new();
        h.app.startup_checked = true;
        h.app.preferences.update_check = Some(false);
        h.app.update_installed = true;
        if state == "quoi-de-neuf" {
            h.app.startup_prompts = true;
            h.app.whats_new = true;
            h.app.whats_new_notes = sample().notes;
        } else if state == "disponible" { h.app.startup_offer = Some(sample()); }
        else {
            h.app.show_about = true;
            match state {
                "telechargement" => h.app.update_phase = Some(tr("Téléchargement…")),
                "verification-sha" => h.app.update_phase = Some(tr("Vérification…")),
                "erreur-reseau" => h.app.update_result = Some(Err("Vérification impossible : connexion interrompue".into())),
                _ => h.app.update_result = Some(Ok(crate::app_update::Status::Current)),
            }
        }
        let output = h.warm(size);
        h.png(size, output, &out.join(format!("avant-{state}.png")));
    }
}
