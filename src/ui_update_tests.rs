use super::*;

fn sample() -> crate::app_update::Latest {
    crate::app_update::Latest {
        version: "v0.14.0".into(),
        page: "https://github.com/HatiFlash/NovyDeck/releases/tag/v0.14.0".into(),
        setup_url: "https://github.com/example/setup.exe".into(),
        sha_url: "https://github.com/example/setup.sha256".into(),
        setup_size: 32_456_789,
        notes: include_str!("../docs/RELEASE-0.14.0.md").into(),
    }
}

#[test]
fn update_notes_parse_language_bullets_bold_headings_and_links() {
    use super::update_ui::{parse_notes, NoteBlock};
    let raw = "## English\n### Highlights\n- **Fast setup:** [Read more](https://example.com) and `plugins`.\n## Français\n### À retenir\n- **Installation rapide :** [Consultez](https://example.com) et `plugins`.";
    let fr = parse_notes(raw, "fr");
    assert!(matches!(&fr[0], NoteBlock::Heading(text) if text == "À retenir"));
    assert!(matches!(&fr[1], NoteBlock::Bullet { title, body } if title == "Installation rapide" && body == "Consultez et plugins."));
    assert!(!format!("{fr:?}").contains("http") && !format!("{fr:?}").contains("**"));
    let en = parse_notes(raw, "en");
    assert!(matches!(&en[1], NoteBlock::Bullet { title, body } if title == "Fast setup" && body == "Read more and plugins."));
}

#[test]
fn update_operation_marks_the_failed_phase_and_completed_steps() {
    use super::update_ui::{step_state, StepState};
    for (phase, expected) in [
        (Some(tr("Téléchargement…")), [StepState::Failed, StepState::Waiting, StepState::Waiting, StepState::Waiting]),
        (Some(tr("Vérification…")), [StepState::Done, StepState::Failed, StepState::Waiting, StepState::Waiting]),
        (Some(tr("Installation…")), [StepState::Done, StepState::Done, StepState::Failed, StepState::Waiting]),
    ] {
        for (index, state) in expected.into_iter().enumerate() {
            assert_eq!(step_state(index, phase, true), state, "phase {phase:?} index {index}");
        }
    }
    assert_eq!(step_state(0, Some(tr("Téléchargement…")), false), StepState::Active);
}

#[test]
fn update_installation_uses_dedicated_dialog_without_duplicate_or_missing_glyphs() {
    fn texts(shape: &egui::Shape, result: &mut Vec<String>) {
        match shape {
            egui::Shape::Text(text) => result.push(text.galley.job.text.clone()),
            egui::Shape::Vec(shapes) => for shape in shapes { texts(shape, result); },
            _ => {}
        }
    }
    for (physical, dpi) in [(vec2(1000.0, 700.0), 1.0), (vec2(1366.0, 768.0), 1.0), (vec2(1920.0, 1080.0), 1.5)] {
        for lang in ["fr", "en"] {
            let size = physical / dpi;
            let mut h = Harness::new();
            h.ctx.set_pixels_per_point(dpi);
            crate::i18n::set_language(lang);
            h.app.show_about = true;
            h.app.update_installing = true;
            h.app.update_target = Some(sample());
            h.app.update_phase = Some(tr("Téléchargement…"));
            let shown = h.warm(size);
            let mut all = Vec::new();
            for clipped in &shown.shapes { texts(&clipped.shape, &mut all); }
            assert!(!all.iter().any(|s| s.contains('□') || s.contains('✦')), "missing icon in {lang}: {all:?}");
            assert_eq!(all.iter().filter(|s| s.as_str() == tr("Téléchargement…")).count(), 1, "duplicate progress: {all:?}");
            assert!(text_position(&shown.shapes, tr("Téléchargement → Vérification → Installation → Relance")).is_none(), "old progress glyph");
            assert!(text_position(&shown.shapes, tr("Vérifier les mises à jour")).is_none(), "about behind operation");
        }
    }
}

#[test]
fn update_dialog_buttons_fit_and_later_dismisses_offline() {
    for (physical, dpi) in [(vec2(1000.0, 700.0), 1.0), (vec2(1366.0, 768.0), 1.0), (vec2(1920.0, 1080.0), 1.5)] {
        for lang in ["fr", "en"] {
            let mut h = Harness::new();
            h.ctx.set_pixels_per_point(dpi);
            crate::i18n::set_language(lang);
            h.app.startup_offer = Some(sample());
            h.app.update_installed = true;
            let size = physical / dpi;
            let shown = h.warm(size);
            let primary = tr("Mettre à jour maintenant");
            let later = tr("Plus tard");
            for label in [primary, later, tr("Voir sur GitHub")] {
                let pos = text_position(&shown.shapes, label).unwrap_or_else(|| panic!("{lang} missing {label} at {physical:?}/{dpi}"));
                assert!(pos.x > 0.0 && pos.x < size.x && pos.y > 0.0 && pos.y < size.y, "{lang} {label} {pos:?}");
            }
            let pos = text_position(&shown.shapes, later).unwrap();
            h.click(size, pos);
            assert!(h.app.startup_offer.is_none(), "{lang} later");
            assert!(h.app.update_confirm.is_none());
            assert!(h.app.update_rx.is_none());
        }
    }
}

#[test]
fn update_primary_opens_the_existing_confirmation_without_network() {
    let size = vec2(1000.0, 700.0);
    let mut h = Harness::new();
    h.app.startup_offer = Some(sample());
    h.app.update_installed = true;
    let shown = h.warm(size);
    let pos = text_position(&shown.shapes, tr("Mettre à jour maintenant")).unwrap();
    h.click(size, pos);
    assert!(h.app.startup_offer.is_none());
    assert_eq!(h.app.update_confirm.as_ref().map(|latest| latest.version.as_str()), Some("v0.14.0"));
    assert!(h.app.update_rx.is_none(), "confirmation does not download");
    let shown = h.warm(size);
    h.click(size, text_position(&shown.shapes, tr("Annuler")).unwrap());
    assert!(h.app.update_confirm.is_none());
    assert!(h.app.update_rx.is_none());
}

#[test]
fn update_after_captures_offline() {
    let Some(out) = std::env::var_os("NOVYDECK_UI_OUTPUT") else { return; };
    let out = PathBuf::from(out);
    let size = vec2(1366.0, 768.0);
    for (state, lang) in [("disponible", "fr"), ("disponible-en", "en"), ("telechargement", "fr"), ("verification-sha", "fr"), ("installation", "fr"), ("erreur-reseau", "fr"), ("erreur-sha", "fr"), ("erreur-installation", "fr"), ("a-jour", "fr"), ("quoi-de-neuf", "fr"), ("quoi-de-neuf-en", "en"), ("quoi-de-neuf-clair-fr", "fr")] {
        let mut h = Harness::new();
        crate::i18n::set_language(lang);
        h.app.startup_checked = true;
        h.app.preferences.update_check = Some(false);
        h.app.update_installed = true;
        // Only the capture fixture simulates an installed 0.13.0; production
        // reads its actual compiled package version.
        h.ctx.data_mut(|d| d.insert_temp(egui::Id::new("novy_test_installed_version"), String::from("0.13.0")));
        h.app.update_target = Some(sample());
        if state == "quoi-de-neuf-clair-fr" {
            h.app.preferences.dark = false;
            theme::apply(&h.ctx, false);
        }
        if state.starts_with("quoi-de-neuf") {
            h.app.startup_prompts = true;
            h.app.whats_new = true;
            h.app.whats_new_notes = sample().notes;
        } else if state.starts_with("disponible") { h.app.startup_offer = Some(sample()); }
        else {
            h.app.show_about = true;
            match state {
                "telechargement" => { h.app.update_installing = true; h.app.update_phase = Some(tr("Téléchargement…")); },
                "verification-sha" => { h.app.update_installing = true; h.app.update_phase = Some(tr("Vérification…")); },
                "installation" => { h.app.update_installing = true; h.app.update_phase = Some(tr("Installation…")); },
                "erreur-reseau" => { h.app.update_installing = true; h.app.update_phase = Some(tr("Téléchargement…")); h.app.update_result = Some(Err("Mise à jour impossible : connexion interrompue".into())); },
                "erreur-sha" => { h.app.update_installing = true; h.app.update_phase = Some(tr("Vérification…")); h.app.update_result = Some(Err("Mise à jour impossible : empreinte invalide".into())); },
                "erreur-installation" => { h.app.update_installing = true; h.app.update_phase = Some(tr("Installation…")); h.app.update_result = Some(Err("Démarrage de l'installateur impossible".into())); },
                _ => h.app.update_result = Some(Ok(crate::app_update::Status::Current)),
            }
        }
        let output = h.warm(size);
        h.png(size, output.clone(), &out.join(format!("apres-{state}.png")));
        if state.starts_with("disponible") {
            assert!(text_position(&output.shapes, "NovyDeck 0.13.0 → 0.14.0").is_some(), "capture must show a genuine upgrade");
        }
        let expected = match state {
            "disponible" | "disponible-en" => Some(tr("Mettre à jour maintenant")),
            "erreur-reseau" => Some(tr("Réessayer")),
            s if s.starts_with("quoi-de-neuf") => Some(tr("C'est parti")),
            _ => None,
        };
        if let Some(label) = expected {
            assert!(text_position(&output.shapes, label).is_some(), "{state}: action {label} invisible");
        }
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
        } else if state.starts_with("disponible") { h.app.startup_offer = Some(sample()); }
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
