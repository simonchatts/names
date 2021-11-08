//! In-memory "database" that manages and caches API results

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use futures::future::LocalBoxFuture;
use sycamore::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::api;
use crate::api::{ApiResult, CountryResult, GenderResult};
use crate::common::*;

/// An in-memory mapping from first name to gender/country API results.
#[derive(Default, Clone)]
pub struct Db(pub Rc<RefCell<HashMap<Name, ApiValue>>>);

/// The API results for a single first name.
#[derive(Default)]
pub struct ApiValue {
    pub gender: Signal<Remote<GenderResult>>,
    pub country: Signal<Remote<Vec<CountryResult>>>,
}

/// Maximum number of queries per batched API call
const API_CHUNKS: usize = 10;

impl Db {
    /// See if any [Name]s have not yet been presented to the API, and if so,
    /// kickoff those API requests. When the results or errors come back, handle
    /// those appropriately.
    pub fn start_any_requests(&self, names: &[Name], error_chan: MpscChannel<ErrMsg>) {
        let mut names_to_query = Vec::new();
        for name in names {
            // Tradeoff: using HashMap::entry would avoid double-lookup, but
            // always requires allocating a fresh key. During interactive edits,
            // repeat lookups can be common. So instead prefer avoiding
            // unncessary allocations.
            let mut map = self.0.borrow_mut();
            // Technically, we should arguably also do a new fetch if there was
            // a previous attempt, but it ended in Remote::Error. But the odds
            // of that happening, and being useful to re-try now, are so low
            // that we just keep things simple, and the user can refresh the
            // page if they want to force re-fetches.
            if map.get(name).is_none() {
                map.insert(name.to_owned(), ApiValue::default());
                names_to_query.push(name.to_owned());
            }
        }

        // If we have at least one name without a cached result, fire off the
        // requests and subsequent handling, in parallel.
        if !names_to_query.is_empty() {
            // Move vec into an Rc, so it can be shared across async contexts.
            let names_to_query = Rc::new(names_to_query);

            // Gender API
            spawn_api_request(
                names_to_query.clone(),
                |names| Box::pin(api::get_genders(names)),
                |api_value| &api_value.gender,
                self.clone(),
                error_chan.clone(),
            );

            // Country API
            spawn_api_request(
                names_to_query,
                |names| Box::pin(api::get_countries(names)),
                |api_value| &api_value.country,
                self.clone(),
                error_chan,
            );
        }
    }
}

/// Common handling: fire off an API request in a fresh async task, and deal
/// with either the success or failure result.
fn spawn_api_request<T: Clone>(
    names_to_query: Rc<Vec<Name>>,
    fetch: impl Fn(&[Name]) -> LocalBoxFuture<ApiResult<T>> + 'static,
    selector: fn(&ApiValue) -> &Signal<Remote<T>>,
    db: Db,
    error_chan: MpscChannel<ErrMsg>,
) {
    // Run the API in a fresh async context.
    spawn_local(async move {
        // Fire off the API request - can do up to 10 names per request.
        // Keep track of the indices into the vector for each query, so
        // we can associate errors with names.
        for chunked_names in names_to_query.chunks(API_CHUNKS) {
            // Remember these particular names, so we can associate an error
            // with them. (We could just save the starting/ending indices into
            // the existing Rc<Vec> but yolo.)
            let saved_names = chunked_names.to_vec();

            // Issue the "fetch" API request and parse the JSON.
            match fetch(chunked_names).await {
                // Success! Update the per-name signals with the result.
                Ok(result) => {
                    let db = db.0.borrow();
                    for (name, item) in result.into_iter() {
                        selector(db.get(&name).unwrap()).set(Remote::Success(item));
                    }
                }

                // Failure
                Err(err) => {
                    // First publish the actual error message...
                    error_chan.send(err.to_string());

                    // ...then set the per-name entry for everything that
                    // was waiting on this to the error state.
                    let db = db.0.borrow();
                    for name in saved_names.iter() {
                        selector(db.get(name).unwrap()).set(Remote::Error);
                    }
                }
            }
        }
    });
}
