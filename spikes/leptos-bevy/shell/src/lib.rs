//! Vỏ Leptos (CSR): giữ `Session<Bars>` có thẩm quyền (ADR 0002), vẽ 2D bằng SVG,
//! bật/tắt bộ hiển thị 3D (Bevy, wasm riêng, tải lười). Hai bộ hiển thị chỉ đọc
//! `State` và phát `Input`; mọi đầu vào đi qua `Session::act` ở đây.
//!
//! Giao tiếp với 3D qua JS (`web/index.html`):
//! - vỏ → 3D: `dyen3d.show(stateJson)` / `dyen3d.hide()`;
//! - 3D → vỏ: Bevy gọi `dyenEmit(inputJson)` → `window.dyenShellInput` do vỏ đăng ký.

use kernel::{Actor, Session};
use kind_bars::{Bars, Input, Relation, State, TICKS};
use leptos::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = dyen3d, js_name = show)]
    fn show3d(state_json: &str);
    #[wasm_bindgen(js_namespace = dyen3d, js_name = hide)]
    fn hide3d();
}

const CONFIG: &str = r#"{"bars":2,"partitions":[3,5],
  "task":{"kind":"compare","left":"1/3","right":"1/5"}}"#;

const W: f64 = 600.0;

fn now_ms() -> u32 {
    web_sys::window()
        .and_then(|w| w.performance())
        .map_or(0, |p| p.now() as u32)
}

#[wasm_bindgen(start)]
pub fn main() {
    leptos::mount::mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let config = serde_json::from_str(CONFIG).expect("cấu hình spike hợp lệ");
    let session = RwSignal::new(Session::<Bars>::new(config).expect("cấu hình spike hợp lệ"));
    let message = RwSignal::new(String::new());
    let three_d = RwSignal::new(false);
    let t0 = now_ms();

    let act = move |input: Input| {
        let mut result = Ok(());
        session.update(|s| result = s.act(now_ms() - t0, Actor::Learner, input));
        message.set(match result {
            Ok(()) => format!("✓ {}", serde_json::to_string(&input).unwrap_or_default()),
            Err(e) => format!("✗ {e}"),
        });
    };

    // 3D → vỏ: nhận Input dạng JSON.
    let inbound = Closure::<dyn Fn(String)>::new(move |json: String| {
        match serde_json::from_str::<Input>(&json) {
            Ok(input) => act(input),
            Err(e) => message.set(format!("✗ đầu vào 3D hỏng: {e}")),
        }
    });
    if let Some(w) = web_sys::window() {
        let _ = js_sys::Reflect::set(&w, &"dyenShellInput".into(), inbound.as_ref());
    }
    inbound.forget();

    // Vỏ → 3D: mỗi khi trạng thái đổi hoặc bật/tắt 3D.
    Effect::new(move |_| {
        let json = session.with(|s| serde_json::to_string(s.state()).unwrap_or_default());
        if three_d.get() {
            show3d(&json);
        } else {
            hide3d();
        }
    });

    let outcome = move || {
        session.with(|s| match kernel::run::<Bars>(s.config(), s.trace()) {
            Ok((_, o)) => serde_json::to_string(&o).unwrap_or_default(),
            Err(e) => e.to_string(),
        })
    };
    let partitions = session.with_untracked(|s| s.config().partitions.clone());

    view! {
        <h1>"1/3 và 1/5, phân số nào lớn hơn?"</h1>
        <p>
            <label>
                <input type="checkbox" on:change=move |ev| three_d.set(event_target_checked(&ev)) />
                " Hiển thị 3D (Bevy)"
            </label>
        </p>
        <p>
            {(0u8..2).flat_map(|bar| {
                partitions.iter().map(move |&parts| view! {
                    <button on:click=move |_| act(Input::Partition { bar, parts })>
                        {format!("Thanh {} → {} phần", bar + 1, parts)}
                    </button>
                }).collect::<Vec<_>>()
            }).collect_view()}
        </p>
        <svg width=W height="260" style="background:#f5efe6">
            {move || session.with(|s| bars_svg(s.state(), act))}
        </svg>
        <p>
            "1/3 ? 1/5: "
            <button on:click=move |_| act(Input::Compare { answer: Relation::Less })>"<"</button>
            <button on:click=move |_| act(Input::Compare { answer: Relation::Equal })>"="</button>
            <button on:click=move |_| act(Input::Compare { answer: Relation::Greater })>">"</button>
            " "
            <button on:click=move |_| act(Input::Done)>"Xong"</button>
        </p>
        <p>{move || message.get()}</p>
        <pre>{outcome}</pre>
    }
}

fn x(ticks: u32) -> f64 {
    f64::from(ticks) * W / f64::from(TICKS)
}

const COLORS: [&str; 4] = ["#e07a5f", "#3d85c6", "#81b29a", "#f2cc8f"];

/// Vẽ thanh (y = 20 + 70·i), vạch chia (bấm để cắt), và thước (y = 190).
fn bars_svg(state: &State, act: impl Fn(Input) + Copy + 'static) -> Vec<AnyView> {
    let ruler = state.ruler().to_vec();
    let mut nodes = Vec::new();
    for bar in 0..state.bar_count() as u8 {
        let y = 20.0 + 70.0 * f64::from(bar);
        for p in state.pieces_of(bar) {
            let placed = ruler.contains(&p.id);
            let id = p.id;
            nodes.push(
                view! {
                    <rect x=x(p.id.start) y=y width=x(p.len) - 2.0 height="40"
                        fill=COLORS[usize::from(bar)] opacity=if placed { "0.3" } else { "1" }
                        stroke="#333" style="cursor:pointer"
                        on:click=move |_| act(if placed { Input::Unplace { piece: id } } else { Input::Place { piece: id } }) />
                }
                .into_any(),
            );
        }
        if let Some(b) = state.bar(bar) {
            for at in b.marks() {
                nodes.push(
                    view! {
                        <line x1=x(at) x2=x(at) y1=y - 6.0 y2=y + 46.0 stroke="#000"
                            stroke-width="4" style="cursor:crosshair"
                            on:click=move |_| act(Input::Cut { bar, at }) />
                    }
                    .into_any(),
                );
            }
        }
    }
    let mut offset = 0;
    for id in ruler {
        if let Some(p) = state.piece(id) {
            nodes.push(
                view! {
                    <rect x=x(offset) y="190" width=x(p.len) - 2.0 height="40"
                        fill=COLORS[usize::from(id.bar)] stroke="#333" style="cursor:pointer"
                        on:click=move |_| act(Input::Unplace { piece: id }) />
                }
                .into_any(),
            );
            offset += p.len;
        }
    }
    nodes.push(view! { <line x1="0" x2=W y1="235" y2="235" stroke="#555" /> }.into_any());
    nodes
}
