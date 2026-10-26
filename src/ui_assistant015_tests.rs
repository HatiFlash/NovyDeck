use super::*;
use egui::{vec2, Rect, Shape};

fn visible_rect(shapes: &[egui::epaint::ClippedShape], label: &str) -> Option<Rect> {
    shapes.iter().find_map(|s| match &s.shape {
        Shape::Text(t) if t.galley.job.text == label && s.clip_rect.contains_rect(t.visual_bounding_rect()) => Some(t.visual_bounding_rect()),
        _ => None,
    })
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
        for text in ["BRANCHE ALORS", "ENFANT REPETER"] {
            assert!(out.shapes.iter().any(|s| matches!(&s.shape, Shape::Text(t) if t.galley.job.text.contains(text))), "enfant {text} absent");
        }
        assert!(visible_rect(&out.shapes, "Ajouter").is_some() || visible_rect(&out.shapes, "Add").is_some());
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
