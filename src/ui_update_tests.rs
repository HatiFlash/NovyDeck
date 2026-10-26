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
fn update_notes_parse_language_bullets_bold_headings_and_links() {
    use super::update_ui::{parse_notes, NoteBlock};
    let raw = "## English\n### Highlights\n- **Fast setup:** [Read more](https://example.com) and `plugins`.\n## Français\n### À retenir\n- **Installation rapide :** [Consultez](https://example.com) et `plugins`.";
    let fr = parse_notes(raw, "fr");
    assert!(matches!(&fr[0], NoteBlock::Heading(text) if text == "À retenir"));
    assert!(matches!(&fr[1], NoteBlock::Bullet { title, body } if title == "Installation rapide :" && body == "Consultez et plugins."));
    assert!(!format!("{fr:?}").contains("http") && !format!("{fr:?}").contains("**"));
    let en = parse_notes(raw, "en");
    assert!(matches!(&en[1], NoteBlock::Bullet { title, body } if title == "Fast setup:" && body == "Read more and plugins."));
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
