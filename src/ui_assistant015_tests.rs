use super::*;
use egui::{vec2, Rect, Shape};

fn visible_rect(shapes: &[egui::epaint::ClippedShape], label: &str) -> Option<Rect> {
    shapes.iter().find_map(|s| match &s.shape {
        Shape::Text(t) if t.galley.job.text == label && s.clip_rect.contains_rect(t.visual_bounding_rect()) => Some(t.visual_bounding_rect()),
        _ => None,
    })
}

fn visible_containing(shapes: &[egui::epaint::ClippedShape], fragment: &str) -> bool {
    shapes.iter().any(|s| matches!(&s.shape, Shape::Text(t)
        if t.galley.job.text.contains(fragment) && s.clip_rect.contains_rect(t.visual_bounding_rect())))
}

fn preview(h: &mut Harness, request: &str, steps: Vec<Step>) {
    h.app.workshop_chat.open = true;
    h.app.workshop_chat.say(crate::i18n::trf("Vous : {request}", &[("request", &request)]));
    h.app.workshop_chat.pending = Some(super::workshop_chat_ui::ChatPreview {
        proposal: crate::local_assistant::Proposal {
            steps,
            notes: vec![],
            missing: vec![],
            status: crate::local_assistant::Status::Reconnue,
        },
        scope: (h.app.config.active, h.app.config.page.clone(), h.app.selected),
        original: None,
        request: request.into(),
        ai: false,
    });
    h.app.show_workflow = true;
}

#[test]
fn assistant015_request_precedes_real_preview_and_add_inserts_only_on_click() {
    for (language, count_label) in [("fr", "3 blocs"), ("en", "3 blocks")] {
        let mut h = Harness::new();
        crate::i18n::set_language(language);
        h.app.config.keys_mut()[h.app.selected].steps.clear();
        preview(&mut h, "test chronologie", vec![
            Step::Delay { ms: 345 },
            Step::IfWindow { title: "Bloc conditionnel".into(), then_steps: vec![Step::Text { text: "BRANCHE ALORS".into() }], else_steps: vec![] },
            Step::Repeat { count: 2, steps: vec![Step::Text { text: "ENFANT REPETER".into() }] },
        ]);
        let size = vec2(1366.0, 768.0);
        let out = h.warm(size);
        let request = if language == "fr" { "Vous : test chronologie" } else { "You: test chronologie" };
        let user = visible_rect(&out.shapes, request).expect("demande visible");
        let count = visible_rect(&out.shapes, count_label).expect("compte visible");
        assert!(user.top() < count.top(), "demande avant proposition");
        // The preview has its own scroll area: the last nested step need not
        // be laid out in the initial viewport, but must become visible on scroll.
        let pointer = count.center() + vec2(50.0, 80.0);
        let mut scrolled = out.clone();
        let mut found = false;
        for _ in 0..8 {
            if visible_containing(&scrolled.shapes, "ENFANT REPETER") {
                found = true;
                break;
            }
            scrolled = h.frame(size, vec![
                Input::PointerMoved(pointer),
                Input::MouseWheel { unit: egui::MouseWheelUnit::Point, delta: vec2(0.0, -75.0), modifiers: Modifiers::NONE },
            ]);
        }
        assert!(found, "dernier enfant de l'aperçu inaccessible après défilement");
        assert!(visible_rect(&scrolled.shapes, "Ajouter").is_some() || visible_rect(&scrolled.shapes, "Add").is_some());
        assert!(h.app.config.keys()[h.app.selected].steps.is_empty());
        let label = if language == "fr" { "Ajouter" } else { "Add" };
        h.click(size, visible_rect(&out.shapes, label).unwrap().center());
        assert_eq!(h.app.config.keys()[h.app.selected].steps.len(), 3);
    }
    crate::i18n::set_language("fr");
}

#[test]
fn assistant015_singular_count_and_choices_fit_fr_en_at_two_dpi() {
    for (physical, dpi) in [(vec2(1366.0, 768.0), 1.0), (vec2(1920.0, 1080.0), 1.5)] {
        for (lang, singular) in [("fr", "1 bloc"), ("en", "1 block")] {
            let mut h = Harness::new();
            h.ctx.set_pixels_per_point(dpi);
            crate::i18n::set_language(lang);
            preview(&mut h, "un bloc", vec![Step::Delay { ms: 321 }]);
            let size = physical / dpi;
            let out = h.warm(size);
            assert!(visible_rect(&out.shapes, singular).is_some(), "{lang} {physical:?} compte");
            let question = if lang == "fr" { "Quelle action souhaites-tu ?" } else { "Which action do you want?" };
            let options = ["Option assez longue numéro un", "Option assez longue numéro deux", "Option assez longue numéro trois"];
            h.app.workshop_chat.pending.as_mut().unwrap().proposal.status = crate::local_assistant::Status::Ambigue {
                question: question.into(), choix: options.iter().map(|s| s.to_string()).collect(), choix_autonomes: true,
            };
            let out = h.warm(size);
            let panel = egui::containers::panel::PanelState::load(&h.ctx, egui::Id::new("workshop_chat_panel")).unwrap().rect;
            for label in std::iter::once(question).chain(options) {
                let rect = visible_rect(&out.shapes, label).unwrap_or_else(|| panic!("{label} invisible at {physical:?} {dpi}"));
                assert!(rect.left() >= panel.left() && rect.right() <= panel.right(), "{label} coupé {rect:?} {panel:?}");
            }
        }
    }
    crate::i18n::set_language("fr");
}

#[test]
fn assistant015_captures_rule_proposal_and_plugin_question() {
    let Some(dir) = std::env::var_os("NOVYDECK_UI_OUTPUT") else { return; };
    let dir = std::path::PathBuf::from(dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (physical, dpi, dark, lang) in [
        (vec2(1366.0, 768.0), 1.0, true, "fr"),
        (vec2(1366.0, 768.0), 1.0, false, "fr"),
        (vec2(1366.0, 768.0), 1.0, true, "en"),
        (vec2(1920.0, 1080.0), 1.5, true, "fr"),
    ] {
        for state in ["proposition", "question"] {
            let mut h = Harness::new();
            crate::i18n::set_language(lang);
            h.app.preferences.language = lang.into();
            h.app.preferences.dark = dark;
            theme::apply(&h.ctx, dark);
            h.ctx.set_pixels_per_point(dpi);
            h.app.show_workflow = true;
            h.app.workshop_chat.open = true;
            h.app.preferences.workshop_chat_mode = "local".into();
            h.app.selected = 0;
            h.app.config.keys_mut()[0].steps.clear();
            let request = if state == "proposition" {
                "ouvre le navigateur brave s'il est pas ouvert et si il deja ouvert ouvre un nouvel onglet"
            } else {
                "utilise SoundPad"
            };
            if state == "proposition" {
                h.app.workshop_chat_send(request.into(), None);
            } else {
                let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/counter.sdPlugin");
                let mut pad = crate::plugins::parse(&folder, &std::fs::read(folder.join("manifest.json")).unwrap()).unwrap();
                pad.name = "SoundPad".into();
                pad.actions[0].name = "Toggle Mute".into();
                pad.actions[1].name = "Record Clip".into();
                let mut knowledge = crate::local_llm::Knowledge::builtin();
                knowledge.plugins = vec![pad];
                let proposal = crate::local_assistant::propose_with(request, &h.app.config, &knowledge);
                h.app.workshop_chat.say(crate::i18n::trf("Vous : {request}", &[("request", &request)]));
                h.app.workshop_chat.pending = Some(super::workshop_chat_ui::ChatPreview {
                    proposal,
                    scope: (h.app.config.active, h.app.config.page.clone(), h.app.selected),
                    original: None,
                    request: request.into(),
                    ai: false,
                });
            }
            let size = physical / dpi;
            let output = h.warm(size);
            let theme_name = if dark { "sombre" } else { "clair" };
            let filename = format!("assistant015-{state}-{}x{}-{theme_name}-{lang}.png", physical.x as u32, physical.y as u32);
            h.png(size, output.clone(), &dir.join(filename));
            let pending = h.app.workshop_chat.pending.as_ref().expect("proposition après capture");
            if state == "proposition" {
                assert!(matches!(pending.proposal.status, crate::local_assistant::Status::Reconnue));
                assert!(matches!(&pending.proposal.steps[..], [Step::IfCondition { then_steps, else_steps, .. }] if then_steps.len() + else_steps.len() >= 3));
                let add = if lang == "fr" { "Ajouter" } else { "Add" };
                assert!(visible_rect(&output.shapes, add).is_some(), "{add} invisible");
            } else {
                assert!(matches!(&pending.proposal.status, crate::local_assistant::Status::Ambigue { choix, .. } if choix.len() == 2 && choix.iter().any(|s| s.contains("Toggle Mute")) && choix.iter().any(|s| s.contains("Record Clip"))));
            }
        }
    }
    crate::i18n::set_language("fr");
}
