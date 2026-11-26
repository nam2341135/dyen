//! Bộ hiển thị loại `bars`: các thanh (SVG), thước, nút chia đều, trả lời so sánh.
//!
//! Toạ độ vẽ là số thực chỉ để hiển thị; vị trí gửi đi luôn là vạch nguyên (ADR 0003).

use kind_bars::{Bars, Config, Input, PieceId, Relation, State, TICKS, Task};
use leptos::prelude::*;

use super::{PlaySignal, footer, start};
use crate::{clock, play::Play};

/// Bề rộng `viewBox`.
const VIEW_W: f64 = 600.0;
/// Toạ độ trái của thanh trong `viewBox`.
const LEFT: f64 = 20.0;
/// Chiều dài một thanh (một đơn vị) trong `viewBox`.
const UNIT: f64 = 560.0;
/// Khoảng cách dọc giữa hai thanh.
const ROW: f64 = STRIP_H + BAR_H + 8.0;
/// Chiều cao một thanh (≥ 24 px ở màn 360 px, ADR 0013).
const BAR_H: f64 = 44.0;
/// Chiều cao dải phía trên thanh: chỗ đặt nút của vạch chia / vết cắt, và (khi cắt tự do)
/// dải "chạm để cắt". Không chồng lên thanh để vùng bấm của mảnh giữ nguyên.
const STRIP_H: f64 = 44.0;
/// Bề rộng tối đa vùng bấm của một nút vạch.
const HANDLE_W: f64 = 44.0;
/// Bán kính nút tròn vẽ trên mỗi vạch.
const KNOB_R: f64 = 8.0;

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

/// Phiên đã kết thúc: người học bấm "Xong", hoặc trả lời so sánh đúng.
///
/// Trả lời so sánh sai thì chưa kết thúc: người học còn sửa được, và cách đánh giá
/// ghi nhận việc sửa (điểm một nửa, `recovered`).
pub fn ended(play: &Play<Bars>) -> bool {
    match play.last_input() {
        Some(Input::Done) => true,
        Some(Input::Compare { .. }) => play.outcome().is_some_and(|o| o.completed),
        _ => false,
    }
}

/// Gửi một đầu vào của người học; bỏ qua nếu phiên đã kết thúc ([`ended`]).
fn act(play: PlaySignal<Bars>, input: Input) {
    if play.with_untracked(ended) {
        return;
    }
    let now = clock::now_ms();
    play.update(|p| {
        p.act_unless(now, input, ended);
    });
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
    let locked = move || play.with(ended);
    let hint = if freehand {
        Some("Chạm vào dải xám phía trên thanh để cắt; chạm nút trên vết cắt để nối lại.")
    } else if partitions.is_empty() {
        None
    } else {
        Some("Chạm nút tròn phía trên vạch chia để cắt; chạm lại để nối.")
    }
    .map(|h| view! { <p class="status">{h}</p> });
    // Không có số phần nào được chia đều thì không vẽ hàng nút trống.
    let partition_rows = if partitions.is_empty() { 0 } else { bars };
    let partition_buttons = (0..partition_rows)
        .map(|bar| {
            let buttons = partitions
                .iter()
                .map(|&parts| {
                    view! {
                        <button
                            on:click=move |_| act(play, Input::Partition { bar, parts })
                            disabled=locked
                        >
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
                    <button
                        on:click=move |_| act(play, Input::Compare { answer: r })
                        disabled=locked
                    >
                        {label}
                    </button>
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
        <svg
            class="bars"
            class:locked=locked
            viewBox=format!("0 0 {VIEW_W} {height}") width="100%">{figure}</svg>
        {hint}
        {partition_buttons}
        {answer}
        <p>
            <button class="primary" on:click=move |_| act(play, Input::Done) disabled=locked>
                "Xong"
            </button>
        </p>
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
    let lines: Vec<(u32, bool)> = {
        let mut v: Vec<(u32, bool)> = cuts.iter().map(|&at| (at, true)).collect();
        v.extend(
            marks
                .into_iter()
                .filter(|m| !cuts.contains(m))
                .map(|at| (at, false)),
        );
        v.sort_unstable();
        v
    };
    let xs: Vec<f64> = lines.iter().map(|&(at, _)| x_of(at)).collect();
    let handles = lines
        .iter()
        .zip(handle_spans(&xs))
        .map(|(&(at, is_cut), span)| {
            let (class, input) = if is_cut {
                ("cut", Input::Join { bar, at })
            } else {
                ("mark", Input::Cut { bar, at })
            };
            handle(play, x_of(at), span, y, class, input)
        })
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
    view! { <g>{strip}{pieces}{handles}</g> }
}

/// Vùng bấm `(trái, phải)` của các nút vạch ở hoành độ `xs` (đã sắp tăng): rộng [`HANDLE_W`],
/// nhưng hai vạch gần nhau thì chia đôi khoảng giữa để vùng bấm không chồng lên nhau.
fn handle_spans(xs: &[f64]) -> Vec<(f64, f64)> {
    let half = HANDLE_W / 2.0;
    xs.iter()
        .enumerate()
        .map(|(i, &x)| {
            let left = i
                .checked_sub(1)
                .and_then(|j| xs.get(j))
                .map_or(x - half, |&p| (x - half).max((p + x) / 2.0));
            let right = xs
                .get(i + 1)
                .map_or(x + half, |&n| (x + half).min((x + n) / 2.0));
            (left, right)
        })
        .collect()
}

/// Một vạch dọc (vạch chia → cắt, vết cắt → nối). Bấm vào nút tròn trên dải phía trên thanh;
/// vùng bấm nằm trọn trong dải nên không lấn vùng bấm của mảnh.
fn handle(
    play: PlaySignal<Bars>,
    x: f64,
    (left, right): (f64, f64),
    y: f64,
    class: &'static str,
    input: Input,
) -> impl IntoView + use<> {
    let knob_y = y - STRIP_H / 2.0;
    view! {
        <g class=class on:click=move |_| act(play, input)>
            <line x1=x y1=knob_y x2=x y2=y + BAR_H + 4.0 />
            <circle class="knob" cx=x cy=knob_y r=KNOB_R />
            <rect class="hit" x=left y=y - STRIP_H width=right - left height=STRIP_H />
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
    use crate::play::Play;

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

    #[test]
    fn vung_bam_du_lon() {
        use crate::kinds::tests::{MIN_TARGET_PX, PHONE_SVG_PX};
        let px = |units: f64| units * PHONE_SVG_PX / VIEW_W;
        assert!(px(BAR_H) >= MIN_TARGET_PX, "thanh cao {:.1} px", px(BAR_H));
        assert!(
            px(STRIP_H) >= MIN_TARGET_PX,
            "dải cao {:.1} px",
            px(STRIP_H)
        );
        assert!(
            px(HANDLE_W) >= MIN_TARGET_PX,
            "nút vạch rộng {:.1} px",
            px(HANDLE_W)
        );
        // Chia đều nhiều nhất: các nút vạch vẫn đủ rộng, và mảnh hẹp nhất cũng vậy.
        let step = TICKS / u32::from(kind_bars::MAX_PARTS);
        let xs: Vec<f64> = (1..u32::from(kind_bars::MAX_PARTS))
            .map(|k| x_of(k * step))
            .collect();
        for (l, r) in handle_spans(&xs) {
            assert!(
                px(r - l) >= MIN_TARGET_PX,
                "nút vạch còn {:.1} px",
                px(r - l)
            );
        }
        let piece = x_of(step) - x_of(0);
        assert!(
            px(piece) >= MIN_TARGET_PX,
            "mảnh 1/{} rộng {:.1} px",
            kind_bars::MAX_PARTS,
            px(piece)
        );
    }

    fn play(task: Task) -> Play<Bars> {
        let config = Config {
            bars: 2,
            freehand: false,
            partitions: vec![2, 3],
            task,
        };
        Play::new(config, 0).expect("cấu hình hợp lệ")
    }

    fn compare() -> Task {
        Task::Compare {
            left: "1/2".parse().expect("phân số"),
            right: "1/3".parse().expect("phân số"),
        }
    }

    #[test]
    fn xong_thi_ket_thuc() {
        let mut p = play(Task::Partition { bar: 0, parts: 2 });
        assert!(!ended(&p));
        p.act(1, Input::Partition { bar: 0, parts: 2 });
        assert!(!ended(&p));
        p.act(2, Input::Done);
        assert!(ended(&p));
    }

    #[test]
    fn so_sanh_sai_van_duoc_tra_loi_lai() {
        let mut p = play(compare());
        p.act(
            1,
            Input::Compare {
                answer: Relation::Less,
            },
        );
        assert!(
            !ended(&p),
            "sai thì còn sửa được (điểm một nửa, `recovered`)"
        );
        p.act(
            2,
            Input::Compare {
                answer: Relation::Greater,
            },
        );
        assert!(ended(&p));
        // Vỏ không phát thêm đầu vào: dấu vết giữ nguyên.
        assert!(!p.act_unless(3, Input::Done, ended));
        assert_eq!(p.steps(), 2);
    }

    #[test]
    fn nut_vach_gan_nhau_khong_chong() {
        let spans = handle_spans(&[100.0, 110.0, 300.0]);
        assert_eq!(spans[0], (100.0 - HANDLE_W / 2.0, 105.0));
        assert_eq!(spans[1], (105.0, 110.0 + HANDLE_W / 2.0));
        assert_eq!(spans[2], (300.0 - HANDLE_W / 2.0, 300.0 + HANDLE_W / 2.0));
        assert!(handle_spans(&[]).is_empty());
    }
}
