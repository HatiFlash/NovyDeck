use super::*;
use crate::device::Command;
use crate::model::{StateDisplay, StateFace};

fn setup() -> (Harness, std::sync::mpsc::Receiver<Command>) {
    let mut h = Harness::new();
    let mut paths = Vec::new();
    for (name, color) in [("on", [30, 200, 90]), ("off", [210, 50, 55])] {
        let path = h.dir.path().join(format!("{name}.png"));
        image::RgbImage::from_pixel(96, 96, image::Rgb(color)).save(&path).unwrap();
        paths.push(path.to_string_lossy().into_owned());
    }
    h.app.config.keys_mut()[0] = Key {
        steps: vec![Step::Toggle { a: vec![], b: vec![] }],
        state_display: Some(StateDisplay {
            toggle: true,
            on: StateFace { image: Some(paths[0].clone()), ..Default::default() },
            off: StateFace { image: Some(paths[1].clone()), ..Default::default() },
            ..Default::default()
        }),
        ..Key::default()
    };
    h.app.applied = h.app.config.clone();
    h.app.sync_state_display();
    let (tx, rx) = std::sync::mpsc::channel();
    h.app.device = tx;
    h.app.wanted = true;
    h.app.gate.connected = true;
    h.app.gate.dirty = false;
    let revision = h.app.gate.request_apply();
    h.app.gate.applied(revision);
    h.app.gate.enabled = true;
    h.app.gate.inspect = false;
    assert_eq!(h.app.gate.decision(), crate::control::Decision::Execute);
    (h, rx)
}

#[test]
fn onoff_key_deployed_to_device_has_no_toggle_badge_and_matches_preview() {
    let (mut h, rx) = setup();
    h.app.upload(false);
    let Some(Command::Update(deployed, _)) = rx.try_iter().find(|c| matches!(c, Command::Update(..))) else { panic!("upload must send an Update") };
    assert!(deployed.keys()[0].runtime_badge.is_none(), "badge sent to device: {:?}", deployed.keys()[0].runtime_badge);
    h.app.applied = deployed;
    h.app.pending = None;
    for _ in 0..2 {
        let _ = rx.try_iter().count();
        h.app.send_state_face(0);
        let jpeg = rx.try_iter().filter_map(|c| match c { Command::KeyImage(0, b) => Some(b), _ => None }).last().expect("face sent");
        let font = crate::key_render::font().unwrap();
        let preview = crate::key_render::for_plugin(&h.app.presented_key(0), h.app.applied.image_size, &font, None, None).unwrap();
        assert!(jpeg == preview, "device and preview must render the same face");
        h.app.toggle_key_face(0);
    }
}

#[test]
fn onoff_press_sends_only_the_new_face_and_no_full_reapply() {
    let (mut h, rx) = setup();
    let font = crate::key_render::font().unwrap();
    let face = |h: &Harness| crate::key_render::for_plugin(&h.app.state_key(0, h.app.applied.keys()[0].clone()), h.app.applied.image_size, &font, None, None).unwrap();
    for round in 0..2 {
        let _ = rx.try_iter().count();
        h.app.execute_physical(0, false);
        let new_face = face(&h);
        // The engine thread reports ToggleVisual asynchronously: let it arrive, then process it.
        for _ in 0..15 { std::thread::sleep(Duration::from_millis(20)); h.app.events(); }
        let sent: Vec<Command> = rx.try_iter().collect();
        assert!(!sent.iter().any(|c| matches!(c, Command::Update(..) | Command::Connect(..))),
            "a full re-apply shows the default face between two states");
        let images: Vec<&Vec<u8>> = sent.iter().filter_map(|c| match c { Command::KeyImage(0, b) => Some(b), _ => None }).collect();
        assert!(images.len() == 1 && *images[0] == new_face, "round {round}: exactly the new state face must be sent, got {} images", images.len());
    }
}

#[test]
fn plain_toggle_without_state_faces_has_no_device_badge_and_no_full_reapply() {
    let (mut h, rx) = setup();
    h.app.config.keys_mut()[0].state_display = None;
    h.app.applied = h.app.config.clone();
    h.app.upload(false);
    let Some(Command::Update(deployed, _)) = rx.try_iter().find(|c| matches!(c, Command::Update(..))) else { panic!("upload must send an Update") };
    assert!(deployed.keys()[0].runtime_badge.is_none(), "badge sent to device: {:?}", deployed.keys()[0].runtime_badge);
    h.app.applied = deployed;
    h.app.pending = None;
    let _ = rx.try_iter().count();
    h.app.execute_physical(0, false);
    for _ in 0..15 { std::thread::sleep(Duration::from_millis(20)); h.app.events(); }
    assert!(!rx.try_iter().any(|c| matches!(c, Command::Update(..) | Command::Connect(..))), "a toggle press must not re-apply the whole deck");
}
