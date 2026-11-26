//! Bộ hiển thị loại `section`: lập phương chiếu xiên (SVG), vị trí bấm để đánh dấu, nút cắt.
//!
//! Toạ độ chiếu là số thực chỉ để vẽ; mọi quyết định do `kind-section` làm bằng số hữu tỉ.

use kind_section::{Config, Input, Section, State, edge_ends, site_points};
use leptos::prelude::*;
use lib_exact::Vec3;

use super::{PlaySignal, act, footer, start};

/// Cạnh khuất (đi qua đỉnh `D` ở góc sau-dưới-trái), vẽ nét đứt.
const HIDDEN: [&str; 3] = ["cd", "da", "dd1"];
/// Tên các điểm đề cho, theo thứ tự trong cấu hình.
const GIVEN_NAMES: [&str; 3] = ["M", "N", "P"];

/// Chiếu xiên một điểm của lập phương đơn vị vào `viewBox="0 0 400 380"`.
pub fn project(p: [f64; 3]) -> (f64, f64) {
    let [x, y, z] = p;
    (40.0 + 220.0 * (x + 0.5 * y), 330.0 - 220.0 * (z + 0.35 * y))
}

fn project_vec(v: Vec3) -> (f64, f64) {
    project([v.x.to_f64(), v.y.to_f64(), v.z.to_f64()])
}

/// Tên hiển thị của một vị trí: `a1` → `A'`, `a1b1` → `A'B'`, `bc` → `BC`.
pub fn site_label(key: &str) -> String {
    key.chars()
        .map(|c| {
            if c == '1' {
                '\''
            } else {
                c.to_ascii_uppercase()
            }
        })
        .collect()
}

/// Vẽ một hoạt động `section`.
pub fn view(config: Config) -> AnyView {
    let sites = match site_points() {
        Ok(s) => s,
        Err(e) => return view! { <p class="error">{format!("Lỗi hình học: {e}")}</p> }.into_any(),
    };
    start::<Section>(config, move |play| body(play, sites))
}

fn body(play: PlaySignal<Section>, sites: Vec<(&'static str, Vec3)>) -> AnyView {
    let figure = move || play.with(|p| figure(play, p.state(), &sites));
    let marked = move || {
        play.with(|p| {
            let m = p.state().marked();
            if m.is_empty() {
                "Chưa đánh dấu vị trí nào.".to_owned()
            } else {
                let names: Vec<String> = m.iter().map(|k| site_label(k)).collect();
                format!("Đang đánh dấu: {}", names.join(", "))
            }
        })
    };
    let status = move || {
        play.with(|p| {
            let s = p.state();
            if s.solved() {
                "Cắt đúng!".to_owned()
            } else if s.finished() {
                "Hết lượt cắt.".to_owned()
            } else {
                format!("Còn {} lượt cắt.", s.attempts_left())
            }
        })
    };
    let finished = move || play.with(|p| p.state().finished());
    view! {
        <svg class="section" viewBox="0 0 400 380" width="100%">{figure}</svg>
        <p>{marked}</p>
        <p>
            <button on:click=move |_| act(play, Input::Cut) disabled=finished>"Cắt"</button>
            " "
            <span class="status">{status}</span>
        </p>
        {footer(play, |p| p.state().finished())}
    }
    .into_any()
}

fn figure(play: PlaySignal<Section>, state: &State, sites: &[(&'static str, Vec3)]) -> AnyView {
    let polygon = state.solved().then(|| {
        let points = polygon_points(
            state
                .section()
                .iter()
                .map(|c| project_vec(c.point))
                .collect(),
        );
        view! { <polygon class="cut" points=points /> }
    });
    let edges = edge_ends()
        .map(|(key, a, b)| {
            let ((x1, y1), (x2, y2)) = (project_vec(a), project_vec(b));
            let class = if HIDDEN.contains(&key) {
                "edge hidden"
            } else {
                "edge"
            };
            view! { <line class=class x1=x1 y1=y1 x2=x2 y2=y2 /> }
        })
        .collect_view();
    let center = project([0.5, 0.5, 0.5]);
    let labels = sites
        .iter()
        .take(8)
        .map(|(key, p)| {
            let (x, y) = outward(project_vec(*p), center, 18.0);
            view! { <text class="vertex" x=x y=y>{site_label(key)}</text> }
        })
        .collect_view();
    let given = state
        .points()
        .iter()
        .zip(GIVEN_NAMES)
        .map(|(p, name)| {
            let (x, y) = project_vec(*p);
            view! {
                <circle class="given" cx=x cy=y r=5.0 />
                <text class="given" x=x + 8.0 y=y - 8.0>{name}</text>
            }
        })
        .collect_view();
    let targets = sites
        .iter()
        .map(|(key, p)| {
            let (x, y) = project_vec(*p);
            let site = (*key).to_owned();
            let is_marked = state.marked().contains(&site);
            let input = if is_marked {
                Input::Unmark { site }
            } else {
                Input::Mark { site }
            };
            view! {
                <circle
                    class=if is_marked { "site marked" } else { "site" }
                    cx=x
                    cy=y
                    r=9.0
                    on:click=move |_| act(play, input.clone())
                >
                    <title>{site_label(key)}</title>
                </circle>
            }
        })
        .collect_view();
    view! { {polygon}{edges}{labels}{given}{targets} }.into_any()
}

/// Dời điểm `p` ra xa `center` một khoảng `d` (để đặt nhãn đỉnh ngoài hình).
fn outward(p: (f64, f64), center: (f64, f64), d: f64) -> (f64, f64) {
    let (dx, dy) = (p.0 - center.0, p.1 - center.1);
    let len = dx.hypot(dy).max(1.0);
    (p.0 + dx / len * d - 5.0, p.1 + dy / len * d + 5.0)
}

/// Chuỗi `points` của đa giác lồi, sắp theo góc quanh trọng tâm (thiết diện là đa giác lồi;
/// phép chiếu song song giữ tính lồi).
fn polygon_points(mut pts: Vec<(f64, f64)>) -> String {
    let n = pts.len().max(1) as f64;
    let cx = pts.iter().map(|p| p.0).sum::<f64>() / n;
    let cy = pts.iter().map(|p| p.1).sum::<f64>() / n;
    pts.sort_by(|a, b| {
        let ta = (a.1 - cy).atan2(a.0 - cx);
        let tb = (b.1 - cy).atan2(b.0 - cx);
        ta.total_cmp(&tb)
    });
    pts.iter()
        .map(|(x, y)| format!("{x:.1},{y:.1}"))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chieu_xien_giu_thu_tu() {
        let (x0, y0) = project([0.0, 0.0, 0.0]);
        let (x1, y1) = project([1.0, 1.0, 1.0]);
        assert!(x0 < x1, "gốc nằm bên trái");
        assert!(y0 > y1, "gốc nằm phía dưới (trục y của SVG hướng xuống)");
        // Mọi đỉnh cách mép viewBox 400×380 ít nhất 15 (chỗ cho nhãn đỉnh).
        for i in 0..8u8 {
            let p = [f64::from(i & 1), f64::from((i >> 1) & 1), f64::from(i >> 2)];
            let (x, y) = project(p);
            assert!((15.0..=385.0).contains(&x), "x = {x} của {p:?}");
            assert!((15.0..=365.0).contains(&y), "y = {y} của {p:?}");
        }
    }

    #[test]
    fn ten_vi_tri() {
        assert_eq!(site_label("a1"), "A'");
        assert_eq!(site_label("a1b1"), "A'B'");
        assert_eq!(site_label("aa1"), "AA'");
        assert_eq!(site_label("bc"), "BC");
    }

    #[test]
    fn da_giac_sap_theo_vong() {
        // Bốn đỉnh hình vuông đưa vào lộn xộn → ra theo vòng (không cắt chéo).
        let s = polygon_points(vec![(0.0, 0.0), (1.0, 1.0), (1.0, 0.0), (0.0, 1.0)]);
        assert_eq!(s, "0.0,0.0 1.0,0.0 1.0,1.0 0.0,1.0");
    }
}
