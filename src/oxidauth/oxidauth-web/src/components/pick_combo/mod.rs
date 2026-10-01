//! One dropdown autocomplete over a fixed catalogue, shared by every pick
//! of a role, permission, or authority. A pick is never typed free-form:
//! the box commits a value only when the text names exactly one choice or
//! a row of the open list is clicked, and the caller still owns the picked
//! signal (clearing it after a grant empties the box again).

use leptos::{ev::KeyboardEvent, prelude::*};

/// One pickable entry: the text shown and filtered, and the id the grant
/// call actually takes — a uuid for roles, the client key for authorities,
/// and both at once for permissions.
#[derive(Clone, Debug, PartialEq)]
pub struct PickChoice {
    pub value: String,
    pub label: String,
}

impl PickChoice {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
        }
    }

    /// The permission case, where the triple is both the id and the text.
    pub fn same(text: impl Into<String>) -> Self {
        let text = text.into();

        Self {
            value: text.clone(),
            label: text,
        }
    }
}

/// The catalogue as the picker sees it. Load state and error text stay the
/// page's own words, so a dead catalogue reads exactly like the disabled
/// option of the select this replaces.
#[derive(Clone, Debug, PartialEq)]
pub enum PickChoices {
    Loading,
    Error(String),
    Ready(Vec<PickChoice>),
}

/// The text box plus the filtered list under it.
///
/// Typing narrows the list and commits live: text that names exactly one
/// choice is as good a pick as a click, anything else holds nothing. The
/// arrow keys walk the list, Enter takes the highlighted row, Escape and
/// blur put the text back to the current pick, and a clear from the owner
/// (`selected` emptied after a grant) empties the box.
#[component]
pub fn PickCombo(
    /// The live catalogue, already minus whatever this row rules out.
    choices: Signal<PickChoices>,
    /// The box's own wording, chosen by each section that mounts it.
    placeholder: &'static str,
    loading_text: &'static str,
    empty_text: &'static str,
    /// The pick in force. Setting it back to the empty string is how the
    /// owner spends it.
    selected: Signal<String>,
    /// Hands over every committed pick, and the empty string the moment
    /// the box no longer names exactly one choice.
    on_pick: Callback<String>,
) -> impl IntoView {
    let (query, set_query) = signal(String::new());
    let (open, set_open) = signal(false);
    let (highlight, set_highlight) = signal(None::<usize>);

    // The value this box last handed over. A `selected` that disagrees
    // with it came from outside — the owner spent the pick — and the box
    // has to follow; a `selected` that agrees is the echo of our own
    // commit and must not touch what the user is typing.
    let (origin, set_origin) = signal(String::new());

    let filtered = Signal::derive(move || {
        match choices.get() {
            PickChoices::Ready(items) => {
                let needle = query.get().to_lowercase();

                items
                    .into_iter()
                    .filter(|item| {
                        item.label
                            .to_lowercase()
                            .contains(&needle)
                    })
                    .collect()
            },
            _ => Vec::new(),
        }
    });

    let commit = Callback::new(move |value: String| {
        set_origin.set(value.clone());
        on_pick.run(value);
    });

    let commit_query = Callback::new(move |_: ()| {
        let text = query
            .get()
            .trim()
            .to_lowercase();

        let value = match choices.get_untracked() {
            PickChoices::Ready(items) if !text.is_empty() => {
                let mut hits = items
                    .iter()
                    .filter(|item| item.label.to_lowercase() == text);

                match (hits.next(), hits.next()) {
                    (Some(one), None) => one.value.clone(),
                    _ => String::new(),
                }
            },
            _ => String::new(),
        };

        commit.run(value);
    });

    Effect::new(move |_| {
        let picked = selected.get();

        if picked == origin.get_untracked() {
            return;
        }

        set_origin.set(picked.clone());

        let want = if picked.is_empty() {
            String::new()
        } else {
            match choices.get_untracked() {
                PickChoices::Ready(items) => {
                    items
                        .iter()
                        .find(|item| item.value == picked)
                        .map(|item| item.label.clone())
                        .unwrap_or_default()
                },
                _ => String::new(),
            }
        };

        if query.get_untracked() != want {
            set_query.set(want);
        }
    });

    let select = Callback::new(move |item: PickChoice| {
        set_query.set(item.label.clone());
        set_open.set(false);
        set_highlight.set(None);
        commit.run(item.value);
    });

    let revert = Callback::new(move |_: ()| {
        let want = match (selected.get_untracked(), choices.get_untracked()) {
            (picked, PickChoices::Ready(items)) if !picked.is_empty() => {
                items
                    .iter()
                    .find(|item| item.value == picked)
                    .map(|item| item.label.clone())
                    .unwrap_or_default()
            },
            _ => String::new(),
        };

        if query.get_untracked() != want {
            set_query.set(want);
        }
    });

    let step = Callback::new(move |dir: isize| {
        let len = filtered.get_untracked().len() as isize;

        if len == 0 {
            return;
        }

        let next = match highlight.get_untracked() {
            Some(idx) => (idx as isize + dir + len) % len,
            None => {
                if dir > 0 {
                    0
                } else {
                    len - 1
                }
            },
        };

        set_highlight.set(Some(next as usize));
    });

    view! {
        <div class="combo">
            <input
                type="text"
                class="combo-input"
                role="combobox"
                aria-expanded=move || open.get().to_string()
                aria-autocomplete="list"
                placeholder=placeholder
                prop:value=move || query.get()
                on:input=move |ev| {
                    set_query.set(event_target_value(&ev));
                    set_open.set(true);
                    set_highlight.set(None);
                    commit_query.run(());
                }

                on:focus=move |_| set_open.set(true)
                on:blur=move |_| {
                    set_open.set(false);
                    set_highlight.set(None);
                    revert.run(());
                }

                on:keydown=move |ev: KeyboardEvent| {
                    match ev.key().as_str() {
                        "ArrowDown" => {
                            ev.prevent_default();
                            if !open.get_untracked() {
                                set_open.set(true);
                            } else {
                                step.run(1);
                            }
                        }
                        "ArrowUp" => {
                            ev.prevent_default();
                            if open.get_untracked() {
                                step.run(-1);
                            }
                        }
                        "Enter" if open.get_untracked() => {
                            ev.prevent_default();

                            if let Some(idx) = highlight.get_untracked()
                                && let Some(item) =
                                    filtered.get_untracked().into_iter().nth(idx)
                            {
                                select.run(item);
                            }
                        }
                        "Escape" if open.get_untracked() => {
                            ev.prevent_default();
                            set_open.set(false);
                            set_highlight.set(None);
                            revert.run(());
                        }
                        _ => {}
                    }
                }
            />

            <Show when=move || open.get()>
                <ul class="combo-list" role="listbox">
                    {move || -> AnyView {
                        match choices.get() {
                            PickChoices::Loading => {
                                let hint = loading_text.to_string();

                                view! { <li class="combo-hint">{hint}</li> }.into_any()
                            },
                            PickChoices::Error(err) => view! {
                                <li class="combo-hint">{err}</li>
                            }
                                .into_any(),
                            PickChoices::Ready(_) => {
                                let items = filtered.get();

                                if items.is_empty() {
                                    let typed = query.get().trim().to_string();
                                    let hint = if typed.is_empty() {
                                        empty_text.to_string()
                                    } else {
                                        format!("no match for \"{typed}\"")
                                    };

                                    view! { <li class="combo-hint">{hint}</li> }.into_any()
                                } else {
                                    items
                                        .into_iter()
                                        .enumerate()
                                        .map(|(idx, item)| {
                                            let label = item.label.clone();

                                            view! {
                                                <li
                                                    class="combo-option"
                                                    role="option"
                                                    attr:aria-selected=move || {
                                                        if highlight.get() == Some(idx) {
                                                            "true"
                                                        } else {
                                                            "false"
                                                        }
                                                    }

                                                    attr:data-active=move || {
                                                        if highlight.get() == Some(idx) {
                                                            "true"
                                                        } else {
                                                            "false"
                                                        }
                                                    }

                                                    on:mousedown=|ev| ev.prevent_default()
                                                    on:click=move |_| select.run(item.clone())
                                                >
                                                    {label}
                                                </li>
                                            }
                                        })
                                        .collect_view()
                                        .into_any()
                                }
                            },
                        }
                    }}
                </ul>
            </Show>
        </div>
    }
}
