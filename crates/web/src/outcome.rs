//! Hiển thị kết quả đánh giá (`Outcome`) với tên nhãn lấy từ nội dung.

use kernel::{Confidence, Outcome};
use leptos::prelude::*;

use crate::library;

/// Điểm phần nghìn thành chuỗi phần trăm, vd. `667` → `"66,7 %"`.
pub fn percent(permille: u16) -> String {
    match permille % 10 {
        0 => format!("{} %", permille / 10),
        d => format!("{},{} %", permille / 10, d),
    }
}

/// Khối kết quả: điểm, đạt/chưa đạt, các phát hiện.
pub fn view(outcome: &Outcome) -> impl IntoView + use<> {
    let tags = library::get().ok().map(|l| l.tags());
    let findings = outcome
        .findings
        .iter()
        .map(|f| {
            let title = tags
                .and_then(|t| t.get(&f.tag))
                .map_or_else(|| f.tag.to_string(), |t| t.title.clone());
            let note = match (f.recovered, f.confidence) {
                (true, _) => " (đã tự sửa)",
                (false, Confidence::Tentative) => " (chưa chắc)",
                (false, Confidence::Strong) => "",
            };
            view! { <li>{title}{note}</li> }
        })
        .collect_view();
    let (class, verdict) = if outcome.completed {
        ("outcome ok", "Đã đạt")
    } else {
        ("outcome", "Chưa đạt")
    };
    view! {
        <section class=class>
            <strong>{verdict}</strong>
            {format!(" · điểm {}", percent(outcome.score.permille()))}
            <ul>{findings}</ul>
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phan_tram() {
        assert_eq!(percent(1000), "100 %");
        assert_eq!(percent(667), "66,7 %");
        assert_eq!(percent(0), "0 %");
    }
}
