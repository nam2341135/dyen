//! Bộ hiển thị loại `bars`: các thanh (SVG), thước, nút chia đều, trả lời so sánh.
//!
//! Toạ độ vẽ là số thực chỉ để hiển thị; vị trí gửi đi luôn là vạch nguyên (ADR 0003).

use kind_bars::{Bars, Config, Input, PieceId, Relation, State, TICKS, Task};
use leptos::prelude::*;

use super::{PlaySignal, act, footer, start};

/// Toạ độ trái của thanh trong `viewBox`.
const LEFT: f64 = 20.0;
/// Chiều dài một thanh (một đơn vị) trong `viewBox`.
const UNIT: f64 = 560.0;
/// Khoảng cách dọc giữa hai thanh.
const ROW: f64 = 70.0;
/// Chiều cao một thanh.
const BAR_H: f64 = 36.0;
/// Chiều cao dải "chạm để cắt" phía trên thanh.
const STRIP_H: f64 = 26.0;

/// Vạch nguyên gần nhất với vị trí `x` trên thanh dài `width` (đơn vị vẽ), kẹp vào `1..TICKS`.
pub fn tick_at(x: f64, width: f64) -> u32 {
    // NaN và số âm thành 0 khi ép kiểu, quá lớn thì bão hoà; sau đó kẹp vào khoảng cắt được.
    let t = (x / width * f64::from(TICKS)).round() as u32;
    t.clamp(1, TICKS - 1)
}

/// Vị trí vẽ của vạch `t`.
fn x_of(t: u32) -> f64 {
    LEFT + f64::from(t) * UNIT / f64::from(TICKS)
}

/// Đỉnh của thanh `bar` (với `bar == số thanh`: chỗ đặt thước).
fn y_of(bar: u8) -> f64 {
    20.0 + STRIP_H + ROW * f64::from(bar)
}

/// Vẽ một hoạt động `bars`.
pub fn view(config: Config) -> AnyView {
    start::<Bars>(config, body)
}

fn body(play: PlaySignal<Bars>) -> AnyView {
    let (bars, partitions, freehand, task) = play.with_untracked(|p| {
        let c = p.config();
        (c.bars, c.partitions.clone(), c.freehand, c.task.clone())
    });
    let height = y_of(bars) + BAR_H + 10.0;
    let figure = move || play.with(|p| figure(play, p.state(), bars, freehand));
    let hint = freehand.then(|| {
        view! { <p class="status">"Chạm vào dải xám phía trên thanh để cắt; chạm vết cắt để nối lại."</p> }
    });
    // Không có số phần nào được chia đều thì không vẽ hàng nút trống.
    let partition_rows = if partitions.is_empty() { 0 } else { bars };
    let partition_buttons = (0..partition_rows)
        .map(|bar| {
            let buttons = partitions
                .iter()
                .map(|&parts| {
                    view! {
                        <button on:click=move |_| act(play, Input::Partition { bar, parts })>
                            {format!("Chia {parts}")}
                        </button>
                    }
                })
                .collect_view();
            view! { <p class="row">{format!("Thanh {}: ", bar + 1)}{buttons}</p> }
        })
        .collect_view();
    let answer = match task {
        Task::Compare { left, right } => {
            let rel = move |r: Relation, label: &'static str| {
                view! {
                    <button on:click=move |_| act(play, Input::Compare { answer: r })>{label}</button>
                }
            };
            Some(view! {
                <p class="row">
                    {format!("{left} ")}
                    {rel(Relation::Less, "<")}
                    {rel(Relation::Equal, "=")}
                    {rel(Relation::Greater, ">")}
                    {format!(" {right}")}
                </p>
            })
        }
        Task::Partition { .. } => None,
    };
    view! {
        <svg class="bars" viewBox=format!("0 0 600 {height}") width="100%">{figure}</svg>
        {hint}
        {partition_buttons}
        {answer}
        <p><button on:click=move |_| act(play, Input::Done)>"Xong"</button></p>
        {footer(play, |p| matches!(p.last_input(), Some(Input::Compare { .. } | Input::Done)))}
    }
    .into_any()
}

/// Nội dung SVG: các thanh với mảnh, vạch chia, vết cắt; thước ở dưới cùng.
fn figure(play: PlaySignal<Bars>, state: &State, bars: u8, freehand: bool) -> AnyView {
    let bar_views = (0..bars)
        .map(|bar| bar_view(play, state, bar, freehand))
        .collect_view();
    let ruler = ruler(play, state, bars);
    view! { {bar_views}{ruler} }.into_any()
}

fn bar_view(
    play: PlaySignal<Bars>,
    state: &State,
    bar: u8,
    freehand: bool,
) -> impl IntoView + use<> {
    let y = y_of(bar);
    let pieces = state
        .pieces_of(bar)
        .into_iter()
        .map(|piece| {
            let placed = state.ruler().contains(&piece.id);
            let input = if placed {
                Input::Unplace { piece: piece.id }
            } else {
                Input::Place { piece: piece.id }
            };
            let x = x_of(piece.id.start);
            view! {
                <rect
                    class=format!("piece c{bar}{}", if placed { " placed" } else { "" })
                    x=x
                    y=y
                    width=x_of(piece.id.start + piece.len) - x
                    height=BAR_H
                    on:click=move |_| act(play, input)
                />
            }
        })
        .collect_view();
    let (marks, cuts) = state
        .bar(bar)
        .map(|b| (b.marks().collect::<Vec<_>>(), b.cuts().collect::<Vec<_>>()))
        .unwrap_or_default();
    let marks = marks
        .into_iter()
        .filter(|m| !cuts.contains(m))
        .map(|at| line(play, at, y, "mark", Input::Cut { bar, at }))
        .collect_view();
    let cuts = cuts
        .into_iter()
        .map(|at| line(play, at, y, "cut", Input::Join { bar, at }))
        .collect_view();
    let strip = freehand.then(|| {
        view! {
            <rect
                class="strip"
                x=LEFT
                y=y - STRIP_H
                width=UNIT
                height=STRIP_H
                on:click=move |ev| {
                    if let Some(at) = strip_tick(&ev) {
                        act(play, Input::Cut { bar, at });
                    }
                }
            >
                <title>"Chạm để cắt"</title>
            </rect>
        }
    });
    view! { <g>{strip}{pieces}{marks}{cuts}</g> }
}

/// Một đường dọc bấm được (vạch chia → cắt, vết cắt → nối), có vùng bấm rộng hơn nét vẽ.
fn line(
    play: PlaySignal<Bars>,
    at: u32,
    y: f64,
    class: &'static str,
    input: Input,
) -> impl IntoView + use<> {
    let x = x_of(at);
    view! {
        <g class=class on:click=move |_| act(play, input)>
            <line x1=x y1=y - 4.0 x2=x y2=y + BAR_H + 4.0 />
            <rect class="hit" x=x - 8.0 y=y - 4.0 width=16.0 height=BAR_H + 8.0 />
        </g>
    }
}

/// Thước: các mảnh đã đặt nối đuôi nhau, trên khung một đơn vị để so.
fn ruler(play: PlaySignal<Bars>, state: &State, bars: u8) -> impl IntoView + use<> {
    let y = y_of(bars) - STRIP_H;
    let mut offset = 0u32;
    let placed = state
        .ruler()
        .iter()
        .filter_map(|id| state.piece(*id))
        .map(|piece| {
            let x = x_of(offset);
            offset += piece.len;
            let w = x_of(offset) - x;
            let id: PieceId = piece.id;
            view! {
                <rect
                    class=format!("piece c{}", id.bar)
                    x=x
                    y=y
                    width=w
                    height=BAR_H
                    on:click=move |_| act(play, Input::Unplace { piece: id })
                />
            }
        })
        .collect_view();
    view! {
        <rect class="ruler" x=LEFT y=y width=UNIT height=BAR_H />
        {placed}
    }
}

/// Vạch được bấm trên dải cắt, tính theo khung thật của phần tử (không phụ thuộc tỉ lệ vẽ).
fn strip_tick(ev: &leptos::ev::MouseEvent) -> Option<u32> {
    use wasm_bindgen::JsCast;
    let el = ev.current_target()?.dyn_into::<web_sys::Element>().ok()?;
    let rect = el.get_bounding_client_rect();
    Some(tick_at(
        f64::from(ev.client_x()) - rect.left(),
        rect.width(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_at_lam_tron_va_kep() {
        let w = 560.0;
        assert_eq!(tick_at(w / 3.0, w), TICKS / 3);
        assert_eq!(tick_at(w / 2.0, w), TICKS / 2);
        assert_eq!(tick_at(0.0, w), 1);
        assert_eq!(tick_at(-5.0, w), 1);
        assert_eq!(tick_at(w, w), TICKS - 1);
        assert_eq!(tick_at(w * 2.0, w), TICKS - 1);
        assert_eq!(tick_at(10.0, 0.0), TICKS - 1);
        assert_eq!(tick_at(f64::NAN, w), 1);
    }
}
