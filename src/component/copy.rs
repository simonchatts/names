//! "Copy to clipboard" button

use crate::prelude::*;

// Component for the "Copy to clipboard" button
#[tracing::instrument(skip_all)]
#[inline_props]
pub fn CopyButton(cx: Scope<'_>, names: UseState<Names>, db: UseRef<Db>) -> Element<'_> {
    // Persistent state: whether or not we are displaying the label saying how
    // many rows have just been copied
    let label = use_state(&cx, Option::<u16>::default);
    // Transient derived state: button disabled or not
    let disabled = label.is_some() || names.is_empty();

    // Onclick handler: do the copy, display the label, and set a timer to remove it
    let onclick = move |_: MouseEvent| {
        let num_rows = copy(names, db);
        label.set(Some(num_rows));
        spawn_local({
            to_owned!(label);
            async move {
                TimeoutFuture::new(3_000).await;
                label.set(None);
            }
        });
    };

    // Optional label
    let label = match label.get() {
        Some(num_rows) => rsx! {
            span {
                class: "copy-label",
                "✓ Copied {num_rows} rows to clipboard"
            }
        },
        _ => rsx! { "" },
    };

    // Overall view
    cx.render(rsx! {
        div {
            class: "copy-button",
            label,
            button {
                class: "btn btn-outline-primary btn-sm",
                disabled: "{disabled}",
                onclick: onclick,
                "Copy M/F column to clipboard"
            }
        }
    })
}

// Copy the M/F column to the clipboard, and return the number of rows
#[tracing::instrument(skip_all)]
fn copy(names: &UseState<Names>, db: &UseRef<Db>) -> u16 {
    // Assemble the plain text string.
    let mut num_rows = 0;
    let mf_col = names
        .iter()
        .map(|n| {
            num_rows += 1;
            match &db.read().get(n).unwrap().gender {
                Remote::Success(gender) => gender.summarised(),
                _ => "?",
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    // Copy the text to the clipboard
    let clipboard = web_sys::window().unwrap().navigator().clipboard().unwrap();
    // The next line fails on Safari when developing with `trunk serve`, since
    // Safari requires a secure context to use this API. But it works fine in
    // production over https, and during development can be tested in Chrome.
    let fut = JsFuture::from(clipboard.write_text(&mf_col));
    spawn_local(async move {
        if let Err(err) = fut.await {
            tracing::error!("Unable to copy to clipboard: {:?}", err);
        }
    });

    // Return the number of rows copied
    num_rows
}
