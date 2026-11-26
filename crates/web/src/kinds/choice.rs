//! Bộ hiển thị loại `choice`: danh sách nút chọn và nút nộp.

use kind_choice::{Choice, Config, Input, State};
use leptos::prelude::*;

use super::{PlaySignal, act, footer, start};

/// Vẽ một hoạt động `choice`.
pub fn view(config: Config) -> AnyView {
    let items: Vec<(String, String)> = config
        .items
        .iter()
        .map(|i| (i.key.clone(), i.text.clone()))
        .collect();
    start::<Choice>(config, move |play| body(play, items))
}

fn body(play: PlaySignal<Choice>, items: Vec<(String, String)>) -> AnyView {
    let buttons = move || {
        play.with(|p| {
            let state = p.state();
            items
                .iter()
                .map(|(key, text)| item_button(play, state, key, text))
                .collect_view()
        })
    };
    let status = move || {
        play.with(|p| {
            let s = p.state();
            if s.solved() {
                "Đúng rồi!".to_owned()
            } else if s.finished() {
                "Hết lượt nộp.".to_owned()
            } else {
                format!("Còn {} lượt nộp.", s.attempts_left())
            }
        })
    };
    let finished = move || play.with(|p| p.state().finished());
    view! {
        <div class="choices">{buttons}</div>
        <p>
            <button on:click=move |_| act(play, Input::Submit) disabled=finished>"Nộp"</button>
            " "
            <span class="status">{status}</span>
        </p>
        {footer(play, |p| p.state().finished())}
    }
    .into_any()
}

fn item_button(
    play: PlaySignal<Choice>,
    state: &State,
    key: &str,
    text: &str,
) -> impl IntoView + use<> {
    let selected = state.selected().iter().any(|k| k == key);
    let input = if selected {
        Input::Deselect {
            key: key.to_owned(),
        }
    } else {
        Input::Select {
            key: key.to_owned(),
        }
    };
    view! {
        <button
            class=if selected { "choice selected" } else { "choice" }
            aria-pressed=selected.to_string()
            disabled=state.finished()
            on:click=move |_| act(play, input.clone())
        >
            {text.to_owned()}
        </button>
    }
}
