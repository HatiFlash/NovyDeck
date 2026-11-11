use super::*;
use crate::model::{PluginBinding, PluginStateTarget, StateDisplay, StateFace};
use crate::plugin_state_display::{self, Observation};
use serde_json::json;
const OWNER: &str = "test.a";
const ACTION: &str = "test.a.switch";
fn setup() -> Harness {
    let mut h = Harness::new();
    let mut paths = Vec::new();
    for (name, color) in [
        ("base", [35, 80, 200]),
        ("on", [30, 200, 90]),
        ("off", [210, 50, 55]),
    ] {
        let path = h.dir.path().join(format!("{name}.png"));
        image::RgbImage::from_pixel(96, 96, image::Rgb(color))
            .save(&path)
            .unwrap();
        paths.push(path.to_string_lossy().into_owned());
    }
    h.app.config.keys_mut()[0] = Key {
        label: "BASE".into(),
        image: paths[0].clone(),
        plugin: Some(PluginBinding {
            plugin: OWNER.into(),
            action: ACTION.into(),
            settings: json!({"id":1}),
            state: 0,
        }),
        plugin_state: Some(PluginStateTarget::Direct),
        state_display: Some(StateDisplay {
            on: StateFace {
                label: Some("ON".into()),
                image: Some(paths[1].clone()),
                ..Default::default()
            },
            off: StateFace {
                label: Some("OFF".into()),
                image: Some(paths[2].clone()),
                ..Default::default()
            },
            ..Default::default()
        }),
        ..Key::default()
    };
    h.app.applied = h.app.config.clone();
    h
}
fn event(h: &Harness, index: usize, state: Option<u64>, origin: Option<&str>) -> serde_json::Value {
    let source = plugin_state_display::source(
        &h.app.applied,
        index,
        h.app.applied.keys()[index].plugin_state.as_ref().unwrap(),
    )
    .unwrap();
    match state {
        Some(state) => {
            let mut v = json!({"event":"setState","context":source.context,"action":source.action,"payload":{"state":state}});
            if let Some(origin) = origin {
                v["payload"]["novydeckStateSource"] = json!(origin);
            }
            v
        }
        None => {
            json!({"event":"novydeckStateUnknown","context":source.context,"action":source.action})
        }
    }
}
#[test]
fn unknown_press_fallback_face_edit_and_reported_state_update_device() {
    let mut h = setup();
    let (tx, rx) = std::sync::mpsc::channel();
    h.app.device = tx;
    h.app.wanted = true;
    h.app.gate.connected = true;
    h.app.gate.dirty = false;
    let off = h.app.presented_key(0);
    assert!(off.image.ends_with("off.png"));
    assert_eq!(off.label, "OFF");
    assert!(off.runtime_badge.is_none());
    h.app.toggle_key_face(0);
    assert_eq!(
        h.app
            .plugin_states
            .lookup(&h.app.config, 0, &h.app.config.keys()[0]),
        Observation::Local(true)
    );
    assert!(h.app.presented_key(0).image.ends_with("on.png"));
    let unknown = event(&h, 0, None, None);
    h.app.plugin_message(OWNER, &unknown);
    assert_eq!(
        h.app
            .plugin_states
            .lookup(&h.app.config, 0, &h.app.config.keys()[0]),
        Observation::Local(true)
    );
    h.app.toggle_key_face(0);
    assert!(h.app.presented_key(0).image.ends_with("off.png"));
    let confirmed = event(&h, 0, Some(1), Some("confirmed"));
    h.app.plugin_message(OWNER, &confirmed);
    assert_eq!(
        h.app
            .plugin_states
            .lookup(&h.app.config, 0, &h.app.config.keys()[0]),
        Observation::Confirmed(true)
    );
    h.app.toggle_key_face(0);
    assert!(
        h.app.presented_key(0).image.ends_with("on.png"),
        "confirmed state must win over local presses"
    );
    let new_icon = h.dir.path().join("updated.png");
    image::RgbImage::from_pixel(96, 96, image::Rgb([90, 40, 210]))
        .save(&new_icon)
        .unwrap();
    h.app.config.keys_mut()[0]
        .state_display
        .as_mut()
        .unwrap()
        .on
        .image = Some(new_icon.to_string_lossy().into_owned());
    let _ = rx.try_iter().count();
    h.app.edited();
    let key = h.app.presented_key(0);
    assert_eq!(key.image, new_icon.to_string_lossy());
    let jpeg = rx
        .try_iter()
        .filter_map(|command| match command {
            crate::device::Command::KeyImage(0, bytes) => Some(bytes),
            _ => None,
        })
        .last()
        .expect("edited icon sent to deck without full apply");
    let font = crate::key_render::font().unwrap();
    assert_eq!(
        jpeg,
        crate::key_render::for_plugin(&key, h.app.config.image_size, &font, None, None).unwrap()
    );
}
#[test]
fn app_state_events_match_preview_and_captured_device_jpeg_without_mutating_config() {
    let mut h = setup();
    let original = serde_json::to_value(&h.app.config).unwrap();
    let (tx, rx) = std::sync::mpsc::channel();
    h.app.device = tx;
    h.app.wanted = true;
    h.app.gate.connected = true;
    h.app.gate.dirty = false;
    for (state, origin, observation, image_name) in [
        (
            Some(1),
            Some("confirmed"),
            Observation::Confirmed(true),
            "on.png",
        ),
        (None, None, Observation::Unknown, "off.png"),
        (Some(0), Some("local"), Observation::Local(false), "off.png"),
        (Some(2), None, Observation::Unknown, "off.png"),
    ] {
        let _ = rx.try_iter().count();
        let msg = event(&h, 0, state, origin);
        h.app.plugin_message(OWNER, &msg);
        assert_eq!(
            h.app
                .plugin_states
                .lookup(&h.app.config, 0, &h.app.config.keys()[0]),
            observation
        );
        let key = h.app.presented_key(0);
        assert!(key.image.ends_with(image_name), "{}", key.image);
        let jpeg = rx
            .try_iter()
            .filter_map(|c| match c {
                crate::device::Command::KeyImage(0, bytes) => Some(bytes),
                _ => None,
            })
            .last()
            .expect("state observation must update the device image");
        assert!(image::load_from_memory(&jpeg).is_ok());
        let font = crate::key_render::font().unwrap();
        let expected =
            crate::key_render::for_plugin(&key, h.app.config.image_size, &font, None, None)
                .unwrap();
        assert_eq!(
            jpeg, expected,
            "device and preview must use the same selected face"
        );
        assert_eq!(serde_json::to_value(&h.app.config).unwrap(), original);
    }
    let msg = event(&h, 0, Some(1), Some("confirmed"));
    h.app.plugin_message("foreign", &msg);
    assert_eq!(
        h.app
            .plugin_states
            .lookup(&h.app.config, 0, &h.app.config.keys()[0]),
        Observation::Unknown
    );
    crate::store::save(&h.app.path, &h.app.config).unwrap();
    let loaded = crate::store::load(&h.app.path).unwrap();
    assert_eq!(
        loaded.keys()[0].plugin_state,
        h.app.config.keys()[0].plugin_state
    );
    assert_eq!(
        loaded.keys()[0].state_display,
        h.app.config.keys()[0].state_display
    );
}
#[test]
fn macro_states_are_distinct_and_unknown_clears_only_selected_instance() {
    let mut h = setup();
    for index in 0..2 {
        let mut key = h.app.config.keys()[0].clone();
        key.plugin = None;
        key.steps = vec![Step::PluginAction {
            plugin: OWNER.into(),
            action: ACTION.into(),
            settings: json!({"id":index}),
            label: format!("Block {index}"),
        }];
        key.plugin_state = Some(PluginStateTarget::Block { path: vec![0] });
        h.app.config.keys_mut()[index] = key;
    }
    h.app.applied = h.app.config.clone();
    for (index, state) in [(0, 1), (1, 0)] {
        let msg = event(&h, index, Some(state), Some("local"));
        h.app.plugin_message(OWNER, &msg);
    }
    assert_eq!(
        h.app
            .plugin_states
            .lookup(&h.app.config, 0, &h.app.config.keys()[0]),
        Observation::Local(true)
    );
    assert_eq!(
        h.app
            .plugin_states
            .lookup(&h.app.config, 1, &h.app.config.keys()[1]),
        Observation::Local(false)
    );
    let msg = event(&h, 0, None, None);
    h.app.plugin_message(OWNER, &msg);
    assert_eq!(
        h.app
            .plugin_states
            .lookup(&h.app.config, 0, &h.app.config.keys()[0]),
        Observation::Unknown
    );
    assert_eq!(
        h.app
            .plugin_states
            .lookup(&h.app.config, 1, &h.app.config.keys()[1]),
        Observation::Local(false)
    );
    assert!(h.app.presented_key(0).image.ends_with("off.png"));
    assert!(h.app.presented_key(0).runtime_badge.is_none());
    assert!(h.app.presented_key(1).image.ends_with("off.png"));
    assert_eq!(h.app.applied.keys()[0].steps, h.app.config.keys()[0].steps);
}
#[test]
fn user_images_and_builtin_faces_take_priority_over_plugin_pixels() {
    let mut h = setup();
    let folder = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/counter.sdPlugin");
    let p = crate::plugins::parse(
        &folder,
        &std::fs::read(folder.join("manifest.json")).unwrap(),
    )
    .unwrap();
    h.app.config.keys_mut()[0].plugin = Some(PluginBinding {
        plugin: p.uuid.clone(),
        action: p.actions[0].uuid.clone(),
        settings: json!({}),
        state: 0,
    });
    h.app.config.keys_mut()[0].plugin_state = None;
    h.app.config.keys_mut()[0].state_display = None;
    h.app.applied = h.app.config.clone();
    let mut manager = crate::plugin_runtime::Manager::new(vec![p]);
    manager.sync(&h.app.applied);
    h.app.plugins = Some(manager);
    let face = PluginFace {
        image_bytes: Some(std::fs::read(&h.app.config.keys()[0].image).unwrap()),
        ..Default::default()
    };
    assert!(h
        .app
        .plugin_image_bytes(0, Some(&face), &h.app.config)
        .is_none());
    assert_eq!(
        h.app
            .plugin_render_key(0, h.app.presented_key(0), &face, &h.app.config)
            .image,
        h.app.config.keys()[0].image
    );
    h.app.config.keys_mut()[0].image.clear();
    h.app.config.keys_mut()[0].state_display = Some(StateDisplay {
        toggle: true,
        on: StateFace {
            image: Some("builtin:speakers".into()),
            ..Default::default()
        },
        ..Default::default()
    });
    h.app.state_values[0] = Some(true);
    let key = h.app.presented_key(0);
    assert!(key.display_kind.is_some());
    assert!(h
        .app
        .plugin_image_bytes(0, Some(&face), &h.app.config)
        .is_none());
    assert_eq!(
        h.app
            .plugin_render_key(0, key.clone(), &face, &h.app.config)
            .display_kind,
        key.display_kind
    );
    h.app.config.keys_mut()[0].state_display = None;
    assert!(
        h.app
            .plugin_image_bytes(0, Some(&face), &h.app.config)
            .is_some(),
        "legacy plugin pixels remain available without a user face"
    );
}
#[test]
fn plugin_messages_never_replace_custom_pixels_or_explicit_empty_state_faces_on_device() {
    let mut h = setup();
    let folder = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/counter.sdPlugin");
    let p = crate::plugins::parse(
        &folder,
        &std::fs::read(folder.join("manifest.json")).unwrap(),
    )
    .unwrap();
    let plugin = p.uuid.clone();
    let action = p.actions[0].uuid.clone();
    h.app.config.keys_mut()[0].plugin = Some(PluginBinding {
        plugin: plugin.clone(),
        action,
        settings: json!({}),
        state: 0,
    });
    h.app.config.keys_mut()[0].plugin_state = None;
    let base_image = h.app.config.keys()[0].image.clone();
    h.app.config.keys_mut()[0].state_display = Some(StateDisplay {
        toggle: true,
        on: StateFace {
            image: Some("".into()),
            label: Some("".into()),
            ..Default::default()
        },
        off: StateFace {
            image: Some(base_image),
            ..Default::default()
        },
        ..Default::default()
    });
    h.app.applied = h.app.config.clone();
    let mut manager = crate::plugin_runtime::Manager::new(vec![p]);
    manager.sync(&h.app.applied);
    let context = manager.visible().values().next().unwrap().context.clone();
    h.app.plugins = Some(manager);
    let (tx, rx) = std::sync::mpsc::channel();
    h.app.device = tx;
    h.app.wanted = true;
    h.app.gate.connected = true;
    h.app.gate.dirty = false;
    let bytes = std::fs::read(&h.app.config.keys()[0].image).unwrap();
    let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);
    for active in [false, true] {
        h.app.state_values[0] = Some(active);
        let expected = h.app.presented_key(0);
        h.app.plugin_message(&plugin,&json!({"event":"setTitle","context":context,"payload":{"title":format!("plugin-{active}")}}));
        let message = json!({"event":"setImage","context":context,"payload":{"image":format!("data:image/png;base64,{encoded}")}});
        h.app.plugin_message(&plugin, &message);
        let jpeg = rx
            .try_iter()
            .filter_map(|c| match c {
                crate::device::Command::KeyImage(0, b) => Some(b),
                _ => None,
            })
            .last()
            .expect("plugin image event updates key");
        let font = crate::key_render::font().unwrap();
        assert!(
            jpeg == crate::key_render::for_plugin(
                &expected,
                h.app.applied.image_size,
                &font,
                None,
                None
            )
            .unwrap(),
            "custom state face must match device"
        );
    }
}
#[test]
fn state_source_picker_and_three_faces_are_visible_in_both_languages_at_hidpi() {
    let size = vec2(1366.0, 768.0);
    for lang in ["fr", "en"] {
        let mut h = setup();
        crate::i18n::set_language(lang);
        h.app.preferences.language = lang.into();
        h.ctx.set_pixels_per_point(1.25);
        h.ctx
            .data_mut(|d| d.insert_persisted(egui::Id::new("inspector_tab"), 1u8));
        h.app.config.keys_mut()[0].plugin_state = None;
        let initial = h.warm(size);
        let origin = text_position(&initial.shapes, crate::i18n::tr("Titre sur la touche"))
            .expect("appearance title control");
        let area = origin + vec2(50.0, 60.0);
        let combo = super::navigation_seek(
            &mut h,
            size,
            crate::i18n::tr("Aucune source plugin"),
            area,
            -70.0,
        );
        h.click(size, combo);
        let menu = h.warm(size);
        let choice = format!("{} · {OWNER} · {ACTION}", crate::i18n::tr("Touche"));
        let option = text_position(&menu.shapes, &choice).expect("explicit source in popup");
        h.click(size, option);
        h.warm(size);
        assert_eq!(
            h.app.config.keys()[0].plugin_state,
            Some(PluginStateTarget::Direct)
        );
        h.app.applied = h.app.config.clone();
        for (name, state, source, label) in [
            ("on", Some(1), Some("confirmed"), "État confirmé"),
            ("off", Some(0), Some("local"), "État local"),
            ("unknown", None, None, "État inconnu"),
        ] {
            let message = event(&h, 0, state, source);
            h.app.plugin_message(OWNER, &message);
            super::navigation_seek(&mut h, size, crate::i18n::tr(label), area, -50.0);
            let output = h.warm(size);
            if let Some(out) = std::env::var_os("NOVYDECK_UI_OUTPUT") {
                h.png(
                    size,
                    output.clone(),
                    &PathBuf::from(out).join(format!("plugin-state-{name}-{lang}.png")),
                );
            }
            assert!(text_position(&output.shapes, crate::i18n::tr(label)).is_some());
        }
        super::navigation_seek(&mut h, size, crate::i18n::tr("Désactivé"), area, -60.0);
        let output = h.warm(size);
        if let Some(out) = std::env::var_os("NOVYDECK_UI_OUTPUT") {
            h.png(
                size,
                output,
                &PathBuf::from(out).join(format!("plugin-state-editor-{lang}.png")),
            );
        }
    }
}

fn trigger_setup() -> (Harness, String) {
    let mut h = Harness::new();
    h.app.config.keys_mut()[0].steps = vec![Step::PluginAction {
        plugin: OWNER.into(),
        action: ACTION.into(),
        settings: json!({}),
        label: "P".into(),
    }];
    h.app.applied = h.app.config.clone();
    h.app.gate.enabled = true;
    h.app.gate.invalid = false;
    h.app.gate.dirty = false;
    let identity = crate::engine::identity(
        &h.app.config.profiles[h.app.config.active].name,
        &h.app.config.page,
        0,
    );
    let context = crate::plugin_runtime::virtual_context(&identity, &[0]);
    (h, context)
}
#[test]
fn plugin_trigger_edge_selects_on_off_idempotently_and_legacy_alternates() {
    let (mut h, context) = trigger_setup();
    h.app.config.keys_mut()[0].steps.push(Step::Toggle {
        a: vec![Step::Notify {
            title: "ON".into(),
            message: "X".into(),
        }],
        b: vec![Step::Notify {
            title: "OFF".into(),
            message: "Y".into(),
        }],
    });
    h.app.config.keys_mut()[0].state_display = Some(crate::model::StateDisplay {
        toggle: true,
        ..Default::default()
    });
    h.app.applied = h.app.config.clone();
    let mut observed = Vec::new();
    for (index, edge) in [
        Some("active"),
        Some("inactive"),
        Some("active"),
        Some("active"),
        None,
        None,
    ]
    .into_iter()
    .enumerate()
    {
        let mut message = json!({"event":"novydeckTrigger", "context":context});
        if let Some(edge) = edge {
            message["payload"] = json!({"edge":edge});
        }
        // The engine serializes macros; wait for the preceding invocation to finish.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        while h
            .app
            .engine
            .running
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            assert!(std::time::Instant::now() < deadline, "macro did not finish");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        h.app.plugin_trigger_log.clear(); // isolate behavior from the 4/s burst limiter
        if edge.is_none() {
            h.app.plugin_trigger_running.clear(); // legacy anti-loop window, not a sensor edge
        }
        h.app.plugin_message(OWNER, &message);
        if let Some(edge) = edge {
            let active = edge == "active";
            assert_eq!(h.app.state_values[0], Some(active), "sensor-selected face");
            assert_eq!(
                h.app.toggle_faces.get(&h.app.toggle_face_id(0)),
                Some(&active)
            );
        }
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        loop {
            for event in h.app.events.try_iter() {
                if let super::Event::Notification(title, _) = event {
                    observed.push(title);
                }
            }
            if observed.len() > index {
                break;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "no ON/OFF action executed: {observed:?}"
            );
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
    assert_eq!(observed, ["ON", "OFF", "ON", "ON", "OFF", "ON"]);
}
fn trigger_count(h: &Harness, context: &str) -> usize {
    h.app.plugin_trigger_log.get(context).map_or(0, |l| l.len())
}
#[test]
fn plugin_trigger_accepts_own_context_and_limits_bursts() {
    let (mut h, context) = trigger_setup();
    let msg = json!({"event":"novydeckTrigger","context":context,"action":ACTION});
    for _ in 0..10 {
        h.app.plugin_message(OWNER, &msg);
        h.app.plugin_trigger_running.clear();
    }
    assert_eq!(
        trigger_count(&h, &context),
        4,
        "anti-rafale : 4 par seconde et par contexte"
    );
}
#[test]
fn plugin_trigger_refuses_foreign_context_plugin_or_action() {
    let (mut h, context) = trigger_setup();
    h.app.plugin_message(
        "autre.plugin",
        &json!({"event":"novydeckTrigger","context":context,"action":ACTION}),
    );
    h.app.plugin_message(
        OWNER,
        &json!({"event":"novydeckTrigger","context":"novydeck-block-00-9","action":ACTION}),
    );
    h.app.plugin_message(
        OWNER,
        &json!({"event":"novydeckTrigger","context":"inconnu"}),
    );
    h.app.plugin_message(
        OWNER,
        &json!({"event":"novydeckTrigger","context":context,"action":"test.a.autre"}),
    );
    assert!(h.app.plugin_trigger_log.is_empty());
}
#[test]
fn plugin_trigger_ignored_when_paused_or_recording() {
    let (mut h, context) = trigger_setup();
    let msg = json!({"event":"novydeckTrigger","context":context,"action":ACTION});
    h.app.gate.enabled = false;
    h.app.plugin_message(OWNER, &msg);
    assert_eq!(trigger_count(&h, &context), 0, "pause globale");
    h.app.gate.enabled = true;
    h.app.trigger_recording = true;
    h.app.plugin_message(OWNER, &msg);
    assert_eq!(trigger_count(&h, &context), 0, "enregistrement");
    h.app.trigger_recording = false;
    h.app.plugin_message(OWNER, &msg);
    assert_eq!(trigger_count(&h, &context), 1);
}

fn log_has(h: &Harness, needle: &str) -> usize {
    h.app.logs.iter().filter(|l| l.contains(needle)).count()
}
#[test]
fn plugin_trigger_refusals_are_logged_once_per_reason_and_window() {
    let (mut h, context) = trigger_setup();
    let bad = json!({"event":"novydeckTrigger","context":"inconnu"});
    for _ in 0..5 {
        h.app.plugin_message(OWNER, &bad);
    }
    assert_eq!(
        log_has(&h, "contexte inconnu"),
        1,
        "une ligne par contexte et par raison"
    );
    h.app.plugin_message(
        "autre.plugin",
        &json!({"event":"novydeckTrigger","context":context,"action":ACTION}),
    );
    assert_eq!(log_has(&h, "plugin ou action différents"), 1);
    let msg = json!({"event":"novydeckTrigger","context":context,"action":ACTION});
    h.app.gate.enabled = false;
    h.app.plugin_message(OWNER, &msg);
    assert_eq!(log_has(&h, "pause globale"), 1);
    h.app.gate.enabled = true;
    h.app.trigger_recording = true;
    h.app.plugin_message(OWNER, &msg);
    assert_eq!(log_has(&h, "enregistrement en cours"), 1);
    h.app.trigger_recording = false;
    h.app.gate.dirty = true;
    h.app.plugin_message(OWNER, &msg);
    assert_eq!(log_has(&h, "édition en cours"), 1);
    h.app.gate.dirty = false;
    let rule = (context.clone(), "pause globale");
    h.app
        .plugin_trigger_refusals
        .insert(rule.clone(), Instant::now() - Duration::from_secs(6));
    h.app.gate.enabled = false;
    h.app.plugin_message(OWNER, &msg);
    assert_eq!(
        log_has(&h, "pause globale"),
        2,
        "à nouveau journalisé après 5 s"
    );
}
#[test]
fn plugin_trigger_loop_is_refused_logged_and_released() {
    let (mut h, context) = trigger_setup();
    let msg = json!({"event":"novydeckTrigger","context":context,"action":ACTION});
    h.app.plugin_message(OWNER, &msg);
    assert_eq!(trigger_count(&h, &context), 1);
    assert!(h.app.plugin_trigger_running.contains_key(&0));
    h.app.plugin_message(OWNER, &msg);
    assert_eq!(trigger_count(&h, &context), 1, "re-déclenchement refusé");
    assert_eq!(log_has(&h, "boucle détectée"), 1);
    h.app
        .plugin_trigger_running
        .insert(0, Instant::now() - Duration::from_secs(3));
    for _ in 0..100 {
        if !h
            .app
            .engine
            .running
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    h.app.plugin_message(OWNER, &msg);
    assert_eq!(
        trigger_count(&h, &context),
        2,
        "autorisé après la fenêtre de 2 s et la fin de l'exécution"
    );
}
#[test]
fn legacy_home_assistant_conditions_are_listed_by_profile_page_and_key() {
    use crate::conditions::Condition;
    let ha = || Condition::HomeAssistantState {
        entity: "light.x".into(),
        attribute: String::new(),
        op: "equals".into(),
        value: "on".into(),
    };
    let mut c = Config::default();
    c.keys_mut()[1].steps = vec![Step::IfCondition {
        condition: Condition::Not {
            condition: Box::new(ha()),
        },
        then_steps: vec![],
        else_steps: vec![],
    }];
    c.keys_mut()[3].state_display = Some(StateDisplay {
        condition: Condition::All {
            conditions: vec![ha()],
        },
        ..Default::default()
    });
    let name = c.profiles[c.active].name.clone();
    let list = crate::conditions::legacy_ha_keys(&c);
    assert_eq!(
        list,
        vec![format!("{name} › touche 2"), format!("{name} › touche 4")]
    );
    assert!(crate::conditions::legacy_ha_keys(&Config::default()).is_empty());
}
