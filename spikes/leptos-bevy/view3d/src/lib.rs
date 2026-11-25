//! Bộ hiển thị 3D (Bevy): chỉ vẽ `State` nhận từ vỏ và phát `Input` khi bấm mảnh.
//! Không giữ trạng thái học tập — mỗi lần nhận `State` thì dựng lại cảnh từ đầu.

use std::cell::RefCell;

use bevy::prelude::*;
use bevy::render::view::Tonemapping;
use kind_bars::{Input, PieceId, State as BarsState, TICKS};
use wasm_bindgen::prelude::*;

thread_local! {
    static INBOX: RefCell<Option<String>> = const { RefCell::new(None) };
}

#[wasm_bindgen]
extern "C" {
    /// 3D → vỏ: gửi một `Input` dạng JSON.
    #[wasm_bindgen(js_name = dyenEmit)]
    fn emit(input_json: &str);
    /// Báo khung hình đầu tiên đã vẽ (đo ADR 0008 điều kiện 2).
    #[wasm_bindgen(js_name = dyenFirstFrame)]
    fn first_frame_done();
}

/// Vỏ → 3D: đặt trạng thái mới; hệ thống `sync` sẽ dựng lại cảnh ở khung kế tiếp.
#[wasm_bindgen]
pub fn push_state(state_json: String) {
    INBOX.with(|i| *i.borrow_mut() = Some(state_json));
}

/// Khởi động Bevy trên `<canvas id="bevy">`.
#[wasm_bindgen]
pub fn start() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                canvas: Some("#bevy".into()),
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(MeshPickingPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (sync, report_first_frame))
        .run();
}

#[derive(Component)]
struct PieceView {
    id: PieceId,
    placed: bool,
}

const LEN: f32 = 6.0;
const COLORS: [(f32, f32, f32); 4] = [
    (0.88, 0.48, 0.37),
    (0.24, 0.52, 0.78),
    (0.51, 0.70, 0.60),
    (0.95, 0.80, 0.56),
];

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(LEN + 1.0, 0.2, 5.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.55, 0.4, 0.25))),
        Transform::from_xyz(0.0, -0.1, 0.0),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 8000.0,
            ..default()
        },
        Transform::from_xyz(3.0, 6.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        Camera3d::default(),
        // Mặc định TonyMcMapFace cần feature `tonemapping_luts` (thêm LUT + zstd); ACES không cần.
        Tonemapping::AcesFitted,
        Transform::from_xyz(0.0, 6.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn ticks(t: u32) -> f32 {
    t as f32 * LEN / TICKS as f32
}

/// Dựng lại các mảnh khi có `State` mới. Thanh i ở z = -1.8 + 1.2·i; thước ở z = 1.8.
fn sync(
    mut commands: Commands,
    old: Query<Entity, With<PieceView>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(json) = INBOX.with(|i| i.borrow_mut().take()) else {
        return;
    };
    let state: BarsState = match serde_json::from_str(&json) {
        Ok(s) => s,
        Err(e) => {
            error!("trạng thái hỏng: {e}");
            return;
        }
    };
    for e in &old {
        commands.entity(e).despawn();
    }
    let ruler = state.ruler();
    let mut spawn = |id: PieceId, len: u32, x0: f32, z: f32, placed: bool, alpha: f32| {
        let (r, g, b) = COLORS[usize::from(id.bar) % COLORS.len()];
        let w = ticks(len) - 0.04;
        commands
            .spawn((
                PieceView { id, placed },
                Mesh3d(meshes.add(Cuboid::new(w, 0.3, 0.6))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgba(r, g, b, alpha),
                    alpha_mode: if alpha < 1.0 {
                        AlphaMode::Blend
                    } else {
                        AlphaMode::Opaque
                    },
                    ..default()
                })),
                Transform::from_xyz(x0 + w / 2.0 - LEN / 2.0, 0.15, z),
            ))
            .observe(on_click);
    };
    for bar in 0..state.bar_count() as u8 {
        let z = -1.8 + 1.2 * f32::from(bar);
        for p in state.pieces_of(bar) {
            let placed = ruler.contains(&p.id);
            spawn(p.id, p.len, ticks(p.id.start), z, placed, if placed { 0.3 } else { 1.0 });
        }
    }
    let mut offset = 0;
    for &id in ruler {
        if let Some(p) = state.piece(id) {
            spawn(id, p.len, ticks(offset), 1.8, true, 1.0);
            offset += p.len;
        }
    }
}

fn on_click(click: On<PointerClick>, pieces: Query<&PieceView>) {
    if let Ok(p) = pieces.get(click.entity) {
        let input = if p.placed {
            Input::Unplace { piece: p.id }
        } else {
            Input::Place { piece: p.id }
        };
        if let Ok(json) = serde_json::to_string(&input) {
            emit(&json);
        }
    }
}

fn report_first_frame(mut done: Local<u32>) {
    // Khung 1 là khung đầu có lệnh vẽ; báo ở khung 2 để chắc nó đã được trình chiếu.
    *done += 1;
    if *done == 2 {
        first_frame_done();
    }
}
