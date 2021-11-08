//! Render global error messages

use std::lazy::SyncOnceCell;

use futures::stream::StreamExt;
use im_rc::OrdMap;

use crate::prelude::*;

/// Internal identifier for one error message
type Id = i32;

/// Number of milliseconds after which an error message is automatically
/// removed.
const AUTO_REMOVAL_TIME: u32 = 10_000;

/// Event loop messages
#[derive(Debug)]
enum Msg {
    Add(ErrMsg),
    Remove(Id),
}

/// Static handle to the event loop coroutine
static CORO: SyncOnceCell<CoroutineHandle<Msg>> = SyncOnceCell::new();

/// Public function for adding a global error message
pub fn add_err_msg(err_msg: ErrMsg) {
    if let Some(coro) = CORO.get() {
        coro.send(Msg::Add(err_msg));
    } else {
        tracing::error!("add_err_msg called with uninitialized coro");
    }
}

/// Error message component
#[tracing::instrument(skip_all)]
pub fn Errors(cx: Scope<'_>) -> Element<'_> {
    // The internal data structure is an [OrdMap] from [Id]s to [ErrMsg]s. The
    // [Id] enables individual error messages to be deleted, and using a
    // [OrdMap] rather than a HashMap means that iterating the structure
    // preserves insertion order, since the [Id] keys monotonically increase
    // over time.
    let next_id = use_state(&cx, Id::default);
    let err_map = use_ref(&cx, OrdMap::<Id, ErrMsg>::default);

    // Event loop
    let coro = use_coroutine(&cx, |mut rx| {
        to_owned![next_id, err_map];
        async move {
            while let Some(msg) = rx.next().await {
                match msg {
                    Msg::Add(err_msg) => {
                        let id = *next_id.current();
                        next_id += 1;
                        err_map.write().insert(id, err_msg);

                        to_owned![err_map];
                        spawn_local(async move {
                            TimeoutFuture::new(AUTO_REMOVAL_TIME).await;
                            err_map.write().remove(&id);
                        });
                    }

                    Msg::Remove(id) => {
                        err_map.write().remove(&id);
                    }
                }
            }
        }
    });

    // Initialize the static handle to this event loop first time through
    if CORO.get().is_none() {
        _ = CORO.set(coro.clone());
    }

    // Component's view
    cx.render(rsx! {
        div {
            class: "errors",
            err_map.read().iter().map(|(&id, err_msg)| {
                rsx! {
                    div {
                        key: "{id}",
                        class: "alert alert-danger", "{err_msg}"
                        button {
                            class: "close",
                            onclick: move |_| coro.send(Msg::Remove(id)),
                            "×"
                        }
                    }
                }
            })
        }
    })
}
