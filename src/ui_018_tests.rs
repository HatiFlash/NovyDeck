//! Drag presentation tests: the binding/swap decision is still owned by gesture::Drag.
use super::*;

#[test]
fn ui018_atelier_cards_slide_pop_and_stop_repainting() {
    let ctx = egui::Context::default();
    let id = egui::Id::new("ui018_atelier_move");
    let a = egui::pos2(40.0, 80.0);
    let b = egui::pos2(40.0, 152.0);
    let mut samples = vec![];
    for (time, target, inserted, enabled) in [
        (0.0, a, false, true), (0.01, b, false, true), (0.07, b, false, true),
        (0.15, b, false, true), (0.24, b, false, true), (0.25, a, false, false),
    ] {
        let mut value = (Vec2::ZERO, 1.0);
        ctx.run(egui::RawInput { time: Some(time), ..Default::default() }, |ctx| {
            value = motion::workshop_card(ctx, id, target, inserted, enabled);
        });
        samples.push(value);
    }
    assert_eq!(samples[0], (Vec2::ZERO, 1.0));
    assert!(samples[1].0.y < samples[2].0.y && samples[2].0.y < samples[3].0.y);
    assert_eq!(samples[4], (Vec2::ZERO, 1.0));
    assert_eq!(samples[5], (Vec2::ZERO, 1.0));
    let pop = egui::Id::new("ui018_atelier_pop");
    let mut first = (Vec2::ZERO, 1.0);
    ctx.run(egui::RawInput { time: Some(0.30), ..Default::default() }, |ctx| { first = motion::workshop_card(ctx, pop, b, true, true); });
    assert!(first.1 < 1.0);
    let mut settled = (Vec2::ZERO, 1.0);
    ctx.run(egui::RawInput { time: Some(0.54), ..Default::default() }, |ctx| { settled = motion::workshop_card(ctx, pop, b, false, true); });
    assert_eq!(settled, (Vec2::ZERO, 1.0));
    assert!(!ctx.has_requested_repaint(), "atelier must stay idle after 220 ms");
}

/// Deterministic headless workshop paint frames; no hardware or app window.
#[test]
#[ignore]
fn ui018_atelier_motion_frames() {
    let dir = std::path::PathBuf::from(std::env::var_os("NOVYDECK_UI_OUTPUT").expect("NOVYDECK_UI_OUTPUT required"));
    std::fs::create_dir_all(&dir).unwrap();
    let ctx = egui::Context::default();
    let size = Vec2::new(360.0, 280.0);
    for (i, ms) in [0_u32, 60, 120, 200, 280].iter().enumerate() {
        let output = ctx.run(egui::RawInput { time: Some(f64::from(*ms) / 1000.0),
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)), ..Default::default() }, |ctx| {
            let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("atelier_frames")));
            painter.rect_filled(egui::Rect::from_min_size(egui::Pos2::ZERO, size), 0.0, Color32::from_rgb(27, 31, 40));
            for (n, base) in [58.0_f32, 119.0, 180.0].iter().enumerate() {
                let target = egui::pos2(34.0, *base + if *ms == 0 || n == 0 { 0.0 } else { 48.0 });
                let (delta, scale) = motion::workshop_card(ctx, egui::Id::new(("atelier_frame", n)), target, false, true);
                let rect = egui::Rect::from_min_size(target + delta, Vec2::new(286.0, 43.0));
                let rect = egui::Rect::from_center_size(rect.center(), rect.size() * scale);
                painter.rect_filled(rect, 8.0, Color32::from_rgb(53 + n as u8 * 14, 67 + n as u8 * 10, 91 + n as u8 * 11));
                painter.rect_filled(egui::Rect::from_min_size(rect.min, Vec2::new(4.0, rect.height())), 0.0, Color32::from_rgb(72, 155, 244));
            }
            if *ms != 0 {
                let slot = egui::Rect::from_min_size(egui::pos2(34.0, 119.0), Vec2::new(286.0, 43.0));
                painter.rect_stroke(slot, 8.0, egui::Stroke::new(2.0, Color32::from_rgb(72, 155, 244)), egui::StrokeKind::Inside);
            }
        });
        let mut png = image::RgbaImage::from_pixel(360, 280, image::Rgba([27, 31, 40, 255]));
        for clipped in &output.shapes {
            let (rect, color) = match &clipped.shape {
                egui::Shape::Rect(s) => (s.rect, s.fill), _ => continue,
            };
            if color == Color32::TRANSPARENT { continue; }
            let rect = rect.intersect(clipped.clip_rect);
            for y in (rect.top().max(0.0) as u32)..(rect.bottom().min(280.0) as u32) {
                for x in (rect.left().max(0.0) as u32)..(rect.right().min(360.0) as u32) {
                    png.put_pixel(x, y, image::Rgba([color.r(), color.g(), color.b(), 255]));
                }
            }
        }
        png.save(dir.join(format!("ui018-atelier-{i:02}.png"))).unwrap();
    }
}

fn visual() -> DragVisual {
    let source = egui::Rect::from_min_size(egui::pos2(26.0, 68.0), Vec2::new(140.0, 56.0));
    DragVisual { item: DragItem::NewFolder, source, center: source.center(), goal: source.center(),
        target: None, phase: 0, started: 0.0, last: 0.0 }
}

#[test]
fn ui018_drag_follows_and_finishes_without_residual_card_or_repaint() {
    let ctx = egui::Context::default();
    let mut v = visual();
    v.goal = egui::pos2(330.0, 110.0);
    assert!(v.advance(0.05, true));
    assert!(v.center.x > v.source.center().x && v.center.x < v.goal.x);
    let flying = ctx.run(egui::RawInput { time: Some(0.05), ..Default::default() }, |ctx| v.paint(ctx, 0.05));
    assert!(flying.shapes.iter().any(|s| matches!(s.shape, egui::Shape::Path(_))), "floating card silhouette");
    let target = egui::Rect::from_center_size(egui::pos2(330.0, 110.0), Vec2::splat(74.0));
    v.target = Some((7, target));
    v.phase = 1;
    v.started = 0.05;
    v.goal = target.center();
    assert!(v.advance(0.20, true));
    assert!(v.center.distance(target.center()) < 1.0);
    let ring = ctx.run(egui::RawInput { time: Some(0.32), ..Default::default() }, |ctx| v.paint(ctx, 0.32));
    assert!(!ring.shapes.is_empty());
    assert!(!v.advance(0.54, true));
    let settled = ctx.run(egui::RawInput { time: Some(0.55), ..Default::default() }, |_| ());
    assert!(settled.shapes.is_empty(), "no residual tooltip after animation");
    assert!(!ctx.has_requested_repaint(), "no repaint at rest");
}

#[test]
fn ui018_cancel_returns_to_origin_and_motion_disabled_is_immediate() {
    let mut v = visual();
    v.center = egui::pos2(310.0, 130.0);
    v.goal = v.source.center();
    v.phase = 2;
    assert!(v.advance(0.10, true));
    assert!(v.center.x < 310.0);
    assert!(!v.advance(0.21, true));
    assert!(!visual().advance(0.0, false));
}

/// PNGs from actual egui shapes at deterministic transition times for visual review.
#[test]
#[ignore]
fn ui018_motion_frames() {
    let dir = std::path::PathBuf::from(std::env::var_os("NOVYDECK_UI_OUTPUT").expect("NOVYDECK_UI_OUTPUT required"));
    std::fs::create_dir_all(&dir).unwrap();
    let ctx = egui::Context::default();
    let size = Vec2::new(480.0, 220.0);
    let target = egui::Rect::from_center_size(egui::pos2(365.0, 105.0), Vec2::splat(78.0));
    let mut v = visual();
    for (stage, times) in [("drag", &[0_u32, 60, 120, 180][..]), ("drop", &[0_u32, 60, 120, 160, 240, 320, 480][..])] {
        if stage == "drop" { v.phase = 1; v.started = 0.20; v.last = 0.20; v.target = Some((7, target)); v.goal = target.center(); }
        for (index, ms) in times.iter().enumerate() {
            let now = if stage == "drop" { 0.20 + f64::from(*ms) / 1000.0 } else { f64::from(*ms) / 1000.0 };
            if stage == "drag" { v.goal = egui::pos2(170.0 + *ms as f32 * 0.9, 80.0); }
            v.advance(now, true);
            let output = ctx.run(egui::RawInput { time: Some(now), screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)), ..Default::default() }, |ctx| {
                let p = ctx.layer_painter(egui::LayerId::new(egui::Order::Background, egui::Id::new("ui018_frame")));
                p.rect_filled(egui::Rect::from_min_size(egui::Pos2::ZERO, size), 0.0, Color32::from_rgb(27, 31, 40));
                p.rect_filled(v.source, 8.0, Color32::from_rgb(55, 62, 79));
                p.rect_filled(target, 12.0, Color32::from_rgb(75, 89, 115));
                if stage == "drag" { p.rect_stroke(target.expand(5.0), 12.0, egui::Stroke::new(3.0, Color32::from_rgb(91, 166, 255)), egui::StrokeKind::Outside); }
                v.paint(ctx, now);
            });
            let mut png = image::RgbaImage::from_pixel(480, 220, image::Rgba([27, 31, 40, 255]));
            for clipped in &output.shapes {
                let (rect, color) = match &clipped.shape {
                    egui::Shape::Rect(s) => (s.rect, s.fill),
                    egui::Shape::Path(s) if s.closed && s.points.len() >= 3 => (egui::Rect::from_points(&s.points), s.fill),
                    _ => continue,
                };
                if color == Color32::TRANSPARENT { continue; }
                let rect = rect.intersect(clipped.clip_rect);
                for y in (rect.top().max(0.0) as u32)..(rect.bottom().min(220.0) as u32) {
                    for x in (rect.left().max(0.0) as u32)..(rect.right().min(480.0) as u32) {
                        let old = png.get_pixel(x, y).0;
                        let a = f32::from(color.a()) / 255.0;
                        png.put_pixel(x, y, image::Rgba([
                            (f32::from(old[0]) * (1.0 - a) + f32::from(color.r()) * a) as u8,
                            (f32::from(old[1]) * (1.0 - a) + f32::from(color.g()) * a) as u8,
                            (f32::from(old[2]) * (1.0 - a) + f32::from(color.b()) * a) as u8, 255]));
                    }
                }
            }
            png.save(dir.join(format!("ui018-{stage}-{index:02}.png"))).unwrap();
        }
    }
}
