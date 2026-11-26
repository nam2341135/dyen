//! Spike "Chiều về trên làng ven sông": cảnh làng 3D dựng hoàn toàn bằng code (xem THIET-KE.md).

mod camera;
mod geom;
mod life;
mod sky;
mod village;
mod world;

use std::cell::RefCell;
use std::sync::Mutex;

use bevy::platform::time::Instant;
use bevy::render::{Render, RenderApp, RenderSystems};

use bevy::prelude::*;
use bevy::window::{PresentMode, PrimaryWindow};

/// Lệnh từ trang web (nút bấm) vào cảnh.
#[derive(Clone, Copy)]
pub enum Cmd {
    Hour(f32),
    Quality,
    Fast,
    Pause,
    Skip,
    Debug(u32),
}

thread_local! {
    static INBOX: RefCell<Vec<Cmd>> = const { RefCell::new(Vec::new()) };
}

/// Gửi lệnh vào cảnh (an toàn gọi từ JS).
pub fn push(cmd: Cmd) {
    INBOX.with(|i| i.borrow_mut().push(cmd));
}

/// Mặt nạ tắt từng phần để đo chi phí (chỉ dùng khi đo).
#[derive(Resource, Default)]
pub struct Dbg(pub u32);

impl Dbg {
    pub fn off(&self, bit: u32) -> bool {
        self.0 & bit != 0
    }
}

#[derive(Resource, Default)]
struct Stats {
    frames: u32,
    acc: f32,
    fps: f32,
    first: Option<f32>,
    started: Option<f32>,
}

// Đo thời gian CPU một khung từ đầu `First` (thế giới chính) tới cuối `Render`
// (gồm cả gửi lệnh WebGL). Không phụ thuộc trình duyệt hãm nhịp khung.
static FRAME_T0: Mutex<Option<Instant>> = Mutex::new(None);
static WORK: Mutex<Vec<f32>> = Mutex::new(Vec::new());

fn mark_start() {
    if let Ok(mut t) = FRAME_T0.lock() {
        *t = Some(Instant::now());
    }
}

fn mark_end() {
    let Some(t0) = FRAME_T0.lock().ok().and_then(|mut t| t.take()) else { return };
    if let Ok(mut w) = WORK.lock() {
        w.push(t0.elapsed().as_secs_f32() * 1000.0);
        if w.len() > 120 {
            w.remove(0);
        }
    }
}

/// Trung vị và trung bình ms CPU/khung của tối đa 120 khung gần nhất; xoá mẫu nếu `reset`.
pub fn work_ms(reset: bool) -> (f32, f32, usize) {
    let Ok(mut w) = WORK.lock() else { return (0.0, 0.0, 0) };
    let mut a = w.clone();
    if reset {
        w.clear();
    }
    if a.is_empty() {
        return (0.0, 0.0, 0);
    }
    a.sort_by(f32::total_cmp);
    (a[a.len() / 2], a.iter().sum::<f32>() / a.len() as f32, a.len())
}

pub fn run() {
    let mut app = App::new();
    app
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Chiều về trên làng ven sông".into(),
                canvas: Some("#bevy".into()),
                fit_canvas_to_parent: true,
                present_mode: PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.6, 0.7, 0.8)))
        .init_resource::<sky::Clock>()
        .init_resource::<sky::Env>()
        .init_resource::<life::Quality>()
        .init_resource::<camera::SkipIntro>()
        .init_resource::<Stats>()
        .init_resource::<Dbg>()
        .add_systems(Startup, (sky::setup, village::setup, camera::spawn))
        .add_systems(PostStartup, life::setup)
        .add_systems(
            Update,
            (
                commands_in,
                sky::tick,
                camera::control,
                sky::update.run_if(|d: Res<Dbg>| !d.off(4)),
                (
                    life::sway.run_if(|d: Res<Dbg>| !d.off(16)),
                    life::rice.run_if(|d: Res<Dbg>| !d.off(2)),
                    life::water.run_if(|d: Res<Dbg>| !d.off(1)),
                    life::lamps,
                    (
                        life::smoke,
                        life::birds,
                        life::boat,
                        life::walkers,
                        life::buffalo,
                        life::kite,
                        life::fireflies,
                    )
                        .run_if(|d: Res<Dbg>| !d.off(32)),
                    dbg_lights,
                ),
                camera::apply_quality,
                hud,
            )
                .chain(),
        )
        .add_systems(First, mark_start);
    app.sub_app_mut(RenderApp).add_systems(Render, mark_end.after(RenderSystems::PostCleanup));
    app.run();
}

fn dbg_lights(dbg: Res<Dbg>, mut q: Query<&mut Visibility, With<PointLight>>) {
    if !dbg.is_changed() {
        return;
    }
    let v = if dbg.off(8) { Visibility::Hidden } else { Visibility::Inherited };
    for mut vis in &mut q {
        *vis = v;
    }
}

fn commands_in(
    keys: Res<ButtonInput<KeyCode>>,
    mut clock: ResMut<sky::Clock>,
    mut quality: ResMut<life::Quality>,
    mut skip: ResMut<camera::SkipIntro>,
    mut dbg: ResMut<Dbg>,
) {
    let mut cmds: Vec<Cmd> = INBOX.with(|i| std::mem::take(&mut *i.borrow_mut()));
    for (k, h) in [(KeyCode::Digit1, 12.0), (KeyCode::Digit2, 17.3), (KeyCode::Digit3, 21.5), (KeyCode::Digit4, 5.9)] {
        if keys.just_pressed(k) {
            cmds.push(Cmd::Hour(h));
        }
    }
    if keys.just_pressed(KeyCode::KeyQ) {
        cmds.push(Cmd::Quality);
    }
    if keys.just_pressed(KeyCode::KeyP) {
        cmds.push(Cmd::Pause);
    }
    if keys.just_pressed(KeyCode::KeyF) {
        cmds.push(Cmd::Fast);
    }
    for c in cmds {
        match c {
            Cmd::Hour(h) => clock.hour = h,
            Cmd::Quality => quality.high = !quality.high,
            Cmd::Fast => clock.fast = !clock.fast,
            Cmd::Pause => clock.paused = !clock.paused,
            Cmd::Skip => skip.0 = true,
            Cmd::Debug(m) => dbg.0 = m,
        }
    }
}

fn hud(
    time: Res<Time>,
    clock: Res<sky::Clock>,
    quality: Res<life::Quality>,
    mut stats: ResMut<Stats>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
) {
    let now = time.elapsed_secs();
    let started = *stats.started.get_or_insert(now);
    if stats.first.is_none() {
        stats.first = Some(now - started);
        #[cfg(target_arch = "wasm32")]
        web::first_frame();
    }
    stats.frames += 1;
    stats.acc += time.delta_secs();
    if stats.acc < 0.5 {
        return;
    }
    stats.fps = stats.frames as f32 / stats.acc;
    stats.frames = 0;
    stats.acc = 0.0;
    let h = clock.hour;
    let text = format!(
        "{:02}:{:02} · {:.0} fps · chất lượng {}{}{}",
        h as u32,
        ((h.fract()) * 60.0) as u32,
        stats.fps,
        if quality.high { "cao" } else { "thấp" },
        if clock.fast { " · tua nhanh" } else { "" },
        if clock.paused { " · dừng" } else { "" },
    );
    #[cfg(target_arch = "wasm32")]
    web::hud(&text);
    #[cfg(not(target_arch = "wasm32"))]
    if let Ok(mut w) = window.single_mut() {
        w.title = format!("Chiều về trên làng ven sông — {text}");
    }
    #[cfg(target_arch = "wasm32")]
    let _ = &mut window;
}

#[cfg(target_arch = "wasm32")]
mod web {
    use wasm_bindgen::prelude::*;

    use super::{Cmd, push};

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_name = dyenHud)]
        pub fn hud(text: &str);
        #[wasm_bindgen(js_name = dyenFirstFrame)]
        pub fn first_frame();
    }

    /// Khởi động cảnh trên `<canvas id="bevy">`.
    #[wasm_bindgen]
    pub fn start() {
        super::run();
    }

    #[wasm_bindgen]
    pub fn set_hour(h: f32) {
        push(Cmd::Hour(h));
    }

    #[wasm_bindgen]
    pub fn toggle_quality() {
        push(Cmd::Quality);
    }

    #[wasm_bindgen]
    pub fn toggle_fast() {
        push(Cmd::Fast);
    }

    #[wasm_bindgen]
    pub fn toggle_pause() {
        push(Cmd::Pause);
    }

    /// Bit: 1 nước, 2 lúa, 4 vòm trời, 8 đèn điểm, 16 cây lay, 32 sinh vật.
    #[wasm_bindgen]
    pub fn set_debug(mask: u32) {
        push(Cmd::Debug(mask));
    }

    /// [trung vị, trung bình, số mẫu] ms CPU/khung; `reset` xoá mẫu cũ.
    #[wasm_bindgen]
    pub fn work_ms(reset: bool) -> Vec<f32> {
        let (m, a, n) = super::work_ms(reset);
        vec![m, a, n as f32]
    }

    #[wasm_bindgen]
    pub fn skip_intro() {
        push(Cmd::Skip);
    }
}
