//! Danh mục: các gói hoạt động theo thứ tự, rồi mục "Khác" cho hoạt động lẻ.

use std::collections::BTreeSet;

use content::{Activity, Library};
use kernel::ActivityId;
use leptos::prelude::*;

/// Các nhóm của danh mục: (tiêu đề nhóm, các hoạt động theo thứ tự trong gói).
pub fn groups(lib: &Library) -> Vec<(String, Vec<ActivityId>)> {
    let mut listed = BTreeSet::new();
    let mut out: Vec<(String, Vec<ActivityId>)> = lib
        .sets()
        .values()
        .map(|set| {
            listed.extend(set.activities.iter().cloned());
            (set.title.clone(), set.activities.clone())
        })
        .collect();
    let rest: Vec<ActivityId> = lib
        .activities()
        .keys()
        .filter(|id| !listed.contains(*id))
        .cloned()
        .collect();
    if !rest.is_empty() {
        out.push(("Khác".to_owned(), rest));
    }
    out
}

/// Vẽ danh mục: mỗi hoạt động là một thẻ có số thứ tự trong gói, tiêu đề và đề bài
/// (cắt còn hai dòng). Bấm thẻ thì đặt `current`.
pub fn view(lib: &'static Library, current: RwSignal<Option<ActivityId>>) -> impl IntoView {
    groups(lib)
        .into_iter()
        .map(|(title, ids)| {
            let links = ids
                .into_iter()
                .filter_map(|id| lib.activities().get(&id))
                .zip(1..)
                .map(|(a, n): (&Activity, u32)| {
                    let id = a.id.clone();
                    view! {
                        <li>
                            <button class="card" on:click=move |_| current.set(Some(id.clone()))>
                                <span class="num" aria-hidden="true">{n}</span>
                                <span class="card-text">
                                    <strong>{a.title.clone()}</strong>
                                    <span class="card-sub">{a.prompt.clone()}</span>
                                </span>
                            </button>
                        </li>
                    }
                })
                .collect_view();
            view! {
                <section>
                    <h2>{title}</h2>
                    <ul class="catalog">{links}</ul>
                </section>
            }
        })
        .collect_view()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn goi_truoc_khong_sot_hoat_dong() {
        let lib = crate::library::load().expect("nội dung nhúng hợp lệ");
        let groups = groups(&lib);
        let all: BTreeSet<_> = groups.iter().flat_map(|(_, ids)| ids.iter()).collect();
        assert_eq!(all.len(), lib.activities().len());
        // Mọi hoạt động hiện có đều thuộc một gói nên không có mục "Khác".
        assert!(groups.iter().all(|(t, _)| t != "Khác"));
    }
}
