//! Release 0.13 screenshots use only generated demo media and isolated app data.
use super::*;

#[test]
fn readme_captures_013_text_overlay_and_action_menu() {
    let Some(out) = std::env::var_os("NOVYDECK_UI_OUTPUT").map(PathBuf::from) else { return; };
    std::fs::create_dir_all(&out).unwrap();
    for lang in ["fr", "en"] {
        let mut h = Harness::new();
        crate::i18n::set_language(lang);
        h.app.preferences.language = lang.into();
        theme::apply(&h.ctx, true);
        let size = vec2(1600.0, 1000.0);
        h.ctx.set_pixels_per_point(1.2);
        let path = h.dir.path().join("demo-cinema.png");
        let mut image = image::RgbImage::new(252, 252);
        for (x, y, pixel) in image.enumerate_pixels_mut() {
            *pixel = image::Rgb([80 + (x / 3) as u8, 130 + (y / 4) as u8, 170]);
        }
        image.save(&path).unwrap();
        h.app.config.keys_mut()[0] = Key {
            label: "Cinema OFF".into(), image: path.to_string_lossy().into_owned(),
            steps: vec![Step::Delay { ms: 100 }], ..Key::default()
        };
        h.app.applied = h.app.config.clone();
        h.ctx.data_mut(|data| data.insert_persisted(egui::Id::new("inspector_tab"), 1u8));
        h.warm(size);
        let label = crate::i18n::tr("Fond et ombre derrière le titre");
        let checkbox = navigation_seek(&mut h, size, label, pos2(1050.0, 890.0), -60.0);
        let output = h.warm(size);
        assert!(fully_visible(&output.shapes, label));
        h.png(size, output, &out.join(format!("v013-text-overlay-on-{lang}.png")));
        h.click(size, checkbox);
        assert!(!h.app.config.keys()[0].label_overlay);
        let output = h.warm(size);
        h.png(size, output, &out.join(format!("v013-text-overlay-off-{lang}.png")));
        let saved = crate::store::load(&h.app.path).unwrap();
        assert!(!saved.keys()[0].label_overlay, "UI choice must survive saving");

        let mut h = Harness::new();
        crate::i18n::set_language(lang);
        h.app.preferences.language = lang.into();
        theme::apply(&h.ctx, true);
        let size = vec2(1366.0, 768.0);
        h.app.config.keys_mut()[0].steps.clear();
        let output = h.warm(size);
        let add = text_position(&output.shapes, crate::i18n::tr("Ajouter")).unwrap();
        h.click(size, add);
        let last = crate::i18n::tr(crate::automation::NAMES.last().unwrap());
        let at = pos2(add.x + 30.0, (add.y + 110.0).min(size.y - 30.0));
        let position = navigation_seek(&mut h, size, last, at, -80.0);
        let output = h.warm(size);
        assert!(fully_visible(&output.shapes, last));
        h.png(size, output, &out.join(format!("v013-action-menu-{lang}.png")));
        h.click(size, position);
        assert!(matches!(h.app.config.keys()[0].steps.as_slice(), [Step::WindowOp { .. }]));
    }
}
