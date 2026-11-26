//! Máy quay: bay mở đầu, sau đó xoay/kéo/phóng bằng chuột và cảm ứng.

use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::input::touch::Touches;
use bevy::pbr::{DistanceFog, FogFalloff};
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::camera::Hdr;
use bevy::render::view::Msaa;
use bevy::render::view::Tonemapping;

use crate::world::height;

#[derive(Clone, Copy)]
struct View {
    focus: Vec3,
    yaw: f32,
    pitch: f32,
    dist: f32,
}

impl View {
    fn eye(&self) -> Vec3 {
        self.focus + self.dist * Vec3::new(self.yaw.sin() * self.pitch.cos(), self.pitch.sin(), self.yaw.cos() * self.pitch.cos())
    }
    fn lerp(a: View, b: View, t: f32) -> View {
        View {
            focus: a.focus.lerp(b.focus, t),
            yaw: a.yaw + (b.yaw - a.yaw) * t,
            pitch: a.pitch + (b.pitch - a.pitch) * t,
            dist: a.dist * (b.dist / a.dist).powf(t),
        }
    }
}

const HOME: View = View { focus: Vec3::new(4.0, 3.0, -14.0), yaw: 0.12, pitch: 0.30, dist: 100.0 };

/// Các điểm dừng của chuyến bay mở đầu (giây, góc nhìn).
const INTRO: [(f32, View); 5] = [
    (0.0, View { focus: Vec3::new(-10.0, 8.0, -40.0), yaw: -0.5, pitch: 0.95, dist: 260.0 }),
    (4.0, View { focus: Vec3::new(-12.0, 5.0, -48.0), yaw: -0.25, pitch: 0.22, dist: 46.0 }),
    (7.5, View { focus: Vec3::new(22.0, 2.0, 18.0), yaw: 1.05, pitch: 0.18, dist: 34.0 }),
    (10.5, View { focus: Vec3::new(8.0, 0.0, 48.0), yaw: 0.6, pitch: 0.42, dist: 30.0 }),
    (14.0, HOME),
];

#[derive(Component)]
pub struct Rig {
    view: View,
    target: View,
    intro: Option<f32>,
    pinch: Option<f32>,
}

/// Ra lệnh bỏ qua phần mở đầu (từ web).
#[derive(Resource, Default)]
pub struct SkipIntro(pub bool);

pub fn spawn(mut commands: Commands) {
    let v = INTRO[0].1;
    commands.spawn((
        Camera3d::default(),
        Hdr,
        Tonemapping::AcesFitted,
        Bloom { intensity: 0.18, ..Bloom::NATURAL },
        Msaa::Sample4,
        Projection::Perspective(PerspectiveProjection { fov: 0.78, near: 0.3, far: 3200.0, ..default() }),
        DistanceFog { color: Color::WHITE, falloff: FogFalloff::Exponential { density: 0.002 }, ..default() },
        Transform::from_translation(v.eye()).looking_at(v.focus, Vec3::Y),
        Rig { view: v, target: v, intro: Some(0.0), pinch: None },
    ));
}

fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn intro_view(t: f32) -> View {
    for w in INTRO.windows(2) {
        let ((t0, a), (t1, b)) = (w[0], w[1]);
        if t <= t1 {
            return View::lerp(a, b, smooth(((t - t0) / (t1 - t0)).clamp(0.0, 1.0)));
        }
    }
    HOME
}

#[allow(clippy::too_many_arguments)] // hệ Bevy
pub fn control(
    time: Res<Time>,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    touches: Res<Touches>,
    mut skip: ResMut<SkipIntro>,
    mut q: Query<(&mut Rig, &mut Transform)>,
) {
    let Ok((mut rig, mut tr)) = q.single_mut() else { return };
    let dt = time.delta_secs().min(0.1);
    let touched = touches.iter().next().is_some();
    let user = buttons.get_pressed().next().is_some() || scroll.delta.y != 0.0 || touched || keys.just_pressed(KeyCode::Space);
    if let Some(t) = rig.intro {
        if user || skip.0 || t > INTRO[INTRO.len() - 1].0 {
            // Ra khỏi mở đầu ở đúng chỗ đang bay tới, không giật.
            let to_home = t > INTRO[INTRO.len() - 1].0 || skip.0 || keys.just_pressed(KeyCode::Space);
            rig.intro = None;
            skip.0 = false;
            rig.target = if to_home { HOME } else { rig.view };
        } else {
            let v = intro_view(t);
            rig.view = v;
            rig.target = v;
            rig.intro = Some(t + dt);
        }
    }
    if rig.intro.is_none() {
        let mut tg = rig.target;
        // Chuột: trái xoay, phải/giữa kéo.
        let d = motion.delta;
        if buttons.pressed(MouseButton::Left) {
            tg.yaw -= d.x * 0.005;
            tg.pitch += d.y * 0.004;
        }
        if buttons.pressed(MouseButton::Right) || buttons.pressed(MouseButton::Middle) {
            let right = Vec3::new(tg.yaw.cos(), 0.0, -tg.yaw.sin());
            let fwd = Vec3::new(-tg.yaw.sin(), 0.0, -tg.yaw.cos());
            let k = tg.dist * 0.0016;
            tg.focus += (-right * d.x + fwd * d.y) * k;
        }
        if scroll.delta.y != 0.0 {
            tg.dist *= (1.0 - scroll.delta.y.signum() * 0.12).clamp(0.5, 1.5);
        }
        // Cảm ứng: một ngón xoay, hai ngón chụm/kéo.
        let ts: Vec<_> = touches.iter().collect();
        if ts.len() == 1 {
            let d = ts[0].delta();
            tg.yaw -= d.x * 0.006;
            tg.pitch += d.y * 0.005;
            rig.pinch = None;
        } else if ts.len() >= 2 {
            let span = ts[0].position().distance(ts[1].position());
            if let Some(prev) = rig.pinch
                && span > 1.0
            {
                tg.dist *= (prev / span).clamp(0.8, 1.25);
            }
            rig.pinch = Some(span);
            let d = (ts[0].delta() + ts[1].delta()) * 0.5;
            let right = Vec3::new(tg.yaw.cos(), 0.0, -tg.yaw.sin());
            let fwd = Vec3::new(-tg.yaw.sin(), 0.0, -tg.yaw.cos());
            tg.focus += (-right * d.x + fwd * d.y) * tg.dist * 0.002;
        } else {
            rig.pinch = None;
        }
        tg.pitch = tg.pitch.clamp(0.06, 1.35);
        tg.dist = tg.dist.clamp(12.0, 280.0);
        tg.focus.x = tg.focus.x.clamp(-120.0, 120.0);
        tg.focus.z = tg.focus.z.clamp(-110.0, 100.0);
        tg.focus.y = height(tg.focus.x, tg.focus.z).max(0.0) + 2.0;
        rig.target = tg;
        // Mượt: tiến dần tới đích.
        let k = 1.0 - (-dt * 7.0).exp();
        let (a, b) = (rig.view, rig.target);
        rig.view = View {
            focus: a.focus.lerp(b.focus, k),
            yaw: a.yaw + (b.yaw - a.yaw) * k,
            pitch: a.pitch + (b.pitch - a.pitch) * k,
            dist: a.dist + (b.dist - a.dist) * k,
        };
    }
    let v = rig.view;
    let mut eye = v.eye();
    // Không chui xuống đất.
    let ground = height(eye.x, eye.z).max(crate::world::WATER_Y) + 1.5;
    if eye.y < ground {
        eye.y = ground;
    }
    *tr = Transform::from_translation(eye).looking_at(v.focus, Vec3::Y);
}

/// Bật/tắt hậu kỳ đắt (bloom, MSAA) theo chất lượng.
pub fn apply_quality(
    mut commands: Commands,
    quality: Res<crate::life::Quality>,
    cam: Query<Entity, With<Rig>>,
    mut light: Query<&mut DirectionalLight>,
) {
    if !quality.is_changed() {
        return;
    }
    let Ok(e) = cam.single() else { return };
    if quality.high {
        commands.entity(e).insert((Bloom { intensity: 0.18, ..Bloom::NATURAL }, Msaa::Sample4));
    } else {
        commands.entity(e).remove::<Bloom>().insert(Msaa::Off);
    }
    for mut l in &mut light {
        l.shadow_maps_enabled = quality.high;
    }
}
