//! Những gì chỉ đổi cách vẽ: làm sáng chỗ được chọn, trời theo giờ, mức chất lượng.

use bevy::camera::Hdr;
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use room_model::{Quality, SlotId};

use super::RoomState;
use super::models::Glow;
use crate::sky::{SkyState, sky_at};

/// Nâng vật đang sáng lên 1,5 cm.
const LIFT: f32 = 0.015;
/// Thời gian chuyển trời sang giá trị mới (giây).
const SKY_FADE: f32 = 2.0;

/// Vật nhìn thấy của một chỗ đặt đồ; `base` là vị trí khi không sáng.
#[derive(Component)]
pub struct SlotVisual {
    /// Chỗ đặt.
    pub id: SlotId,
    /// Transform gốc.
    pub base: Transform,
    /// Màu tự sáng khi được chọn.
    pub glow: LinearRgba,
}

/// Một tấm trời sau cửa sổ (`top`: nửa trên, đỉnh trời).
#[derive(Component)]
pub struct SkyPane {
    /// Nửa trên.
    pub top: bool,
}

/// Một ngôi sao hoặc đèn thành phố xa sau cửa sổ (hiện về đêm).
#[derive(Component)]
pub struct Star;

/// Ánh trời qua cửa sổ.
#[derive(Component)]
pub struct SunLight;

/// Đèn bàn.
#[derive(Component)]
pub struct DeskLampLight;

/// Làm sáng chỗ `Ctl::highlighted`: nâng 1,5 cm và tăng `emissive` trên mọi vật liệu riêng
/// của vật đó (`Glow`); trả chỗ cũ về như trước.
pub fn highlight(
    room: Res<RoomState>,
    mut last: Local<Option<SlotId>>,
    mut q: Query<(&SlotVisual, &mut Transform, &Glow)>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    let now = room.ctl.highlighted();
    if *last == now {
        return;
    }
    for (v, mut t, g) in &mut q {
        let on = Some(v.id) == now;
        if !on && Some(v.id) != *last {
            continue;
        }
        *t = v.base;
        if on {
            t.translation.y += LIFT;
        }
        for h in &g.0 {
            if let Some(mut mat) = mats.get_mut(h) {
                mat.emissive = if on { v.glow } else { LinearRgba::BLACK };
            }
        }
    }
    *last = now;
}

/// Đang chuyển trời từ `from` sang `to`, bắt đầu lúc `t0`.
#[derive(Clone, Copy)]
pub struct SkyFade {
    from: SkyState,
    to: SkyState,
    t0: f32,
}

fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

fn lerp_sky(a: SkyState, b: SkyState, t: f32) -> SkyState {
    let f = |x: f32, y: f32| x + (y - x) * t;
    SkyState {
        top: lerp3(a.top, b.top, t),
        bottom: lerp3(a.bottom, b.bottom, t),
        sun_color: lerp3(a.sun_color, b.sun_color, t),
        sun_lux: f(a.sun_lux, b.sun_lux),
        lamp_lumens: f(a.lamp_lumens, b.lamp_lumens),
        stars: f(a.stars, b.stars),
    }
}

fn linear(c: [f32; 3]) -> Color {
    Color::linear_rgb(c[0], c[1], c[2])
}

/// Trời theo `Ctl::hour`: màu hai tấm trời, ánh trời, đèn bàn, sao. Đổi giờ thì chuyển dần
/// trong 2 s (giảm chuyển động: đổi ngay). Lần đầu đặt ngay.
#[allow(clippy::too_many_arguments)] // Một hệ thống Bevy đọc nhiều loại thực thể; tách ra không gọn hơn.
pub fn sky(
    room: Res<RoomState>,
    time: Res<Time>,
    mut fade: Local<Option<SkyFade>>,
    panes: Query<(&SkyPane, &MeshMaterial3d<StandardMaterial>)>,
    stars: Query<&MeshMaterial3d<StandardMaterial>, With<Star>>,
    mut sun: Query<&mut DirectionalLight, With<SunLight>>,
    mut lamp: Query<&mut PointLight, With<DeskLampLight>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    let now = time.elapsed_secs();
    let target = sky_at(room.ctl.hour());
    let f = match *fade {
        None => SkyFade {
            from: target,
            to: target,
            t0: now - SKY_FADE,
        },
        Some(f) if f.to != target => {
            let cur = lerp_sky(f.from, f.to, ((now - f.t0) / SKY_FADE).clamp(0.0, 1.0));
            let t0 = if room.ctl.reduced() {
                now - SKY_FADE
            } else {
                now
            };
            SkyFade {
                from: cur,
                to: target,
                t0,
            }
        }
        Some(f) => {
            // Đã tới đích từ khung trước: không có gì phải đổi.
            if now - f.t0 > SKY_FADE + 0.1 {
                return;
            }
            f
        }
    };
    *fade = Some(f);
    let s = lerp_sky(f.from, f.to, ((now - f.t0) / SKY_FADE).clamp(0.0, 1.0));
    for (p, m) in &panes {
        if let Some(mut mat) = mats.get_mut(&m.0) {
            mat.base_color = linear(if p.top { s.top } else { s.bottom });
        }
    }
    // Sao và đèn thành phố dùng chung vài vật liệu; mỗi vật liệu đặt một lần.
    let mut seen: Vec<AssetId<StandardMaterial>> = Vec::new();
    for m in &stars {
        if seen.contains(&m.0.id()) {
            continue;
        }
        seen.push(m.0.id());
        if let Some(mut mat) = mats.get_mut(&m.0) {
            mat.base_color.set_alpha(s.stars);
        }
    }
    for mut l in &mut sun {
        l.color = linear(s.sun_color);
        l.illuminance = s.sun_lux;
    }
    for mut l in &mut lamp {
        l.intensity = s.lamp_lumens;
    }
}

/// Mức chất lượng của `Ctl`:
/// - `High`: bloom nhẹ, MSAA 4×, bóng cho đèn bàn;
/// - `Low`: không bloom (và không HDR), MSAA tắt, không bóng. Độ phân giải 0,75× do vỏ làm bằng CSS
///   (lớp `thap` trên canvas), không ở đây.
pub fn quality(
    room: Res<RoomState>,
    mut last: Local<Option<Quality>>,
    mut commands: Commands,
    cams: Query<Entity, With<Camera3d>>,
    mut lamps: Query<&mut PointLight, With<DeskLampLight>>,
) {
    let q = room.ctl.quality();
    if *last == Some(q) {
        return;
    }
    *last = Some(q);
    let high = q == Quality::High;
    for cam in &cams {
        let mut e = commands.entity(cam);
        if high {
            e.insert((
                Msaa::Sample4,
                Bloom {
                    intensity: 0.08,
                    ..Bloom::NATURAL
                },
            ));
        } else {
            e.insert(Msaa::Off).remove::<(Bloom, Hdr)>();
        }
    }
    for mut l in &mut lamps {
        l.shadow_maps_enabled = high;
    }
    // Độ phân giải không đổi ở đây: trên web `scale_factor_override` không đổi bộ đệm canvas
    // (winit đặt = cỡ CSS × devicePixelRatio). Vỏ thu nhỏ canvas bằng lớp CSS `thap`.
}
