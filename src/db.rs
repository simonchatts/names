//! In-memory "database" that manages and caches API results

use std::rc::Rc;

use futures::future::LocalBoxFuture;

use crate::prelude::*;

/// Maximum number of queries per batched API call
const API_CHUNKS: usize = 10;

/// See if any [Names] have not yet been presented to the API, and if so,
/// kickoff those API requests. When the results or errors come back, handle
/// those appropriately.
pub fn start_any_requests(names: &Names, db: UseRef<Db>) {
    let mut names_to_query = Vec::new();
    for name in names {
        // Technically, we should arguably also do a new fetch if there was
        // a previous attempt, but it ended in Remote::Error. But the odds
        // of that happening, and being useful to re-try now, are so low
        // that we just keep things simple, and the user can refresh the
        // page if they want to force re-fetches.
        if db.read().get(name).is_none() {
            db.write().insert(name.to_owned(), AllResults::default());
            names_to_query.push(name.to_owned());
        }
    }

    // If we have no uncached names, nothing to do.
    if names_to_query.is_empty() {
        return;
    }

    // Move vec into an Rc, so it can be shared across the two parallel async
    // API queries.
    let names_to_query = Rc::new(names_to_query);

    // In parallel, kick off the Gender API request...
    spawn_api_request(
        names_to_query.clone(),
        db.clone(),
        |names| Box::pin(get_genders(names)),
        |api_value| &mut api_value.gender,
    );

    // ...and the Country API request.
    spawn_api_request(
        names_to_query,
        db,
        |names| Box::pin(get_countries(names)),
        |api_value| &mut api_value.country,
    );
}

/// Common handling: fire off an API request in a fresh async task, and deal
/// with either the success or failure result.
fn spawn_api_request<T: Clone + 'static>(
    names_to_query: Rc<Names>,
    db: UseRef<Db>,
    fetch: impl Fn(&[Name]) -> LocalBoxFuture<'_, ApiResult<T>> + 'static,
    selector: fn(&mut AllResults) -> &mut Remote<T>,
) {
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
            let result = fetch(chunked_names).await;
            let mut db = db.write();
            match result {
                // Success! Update the per-name signals with the result.
                Ok(result) => {
                    for (name, item) in result.into_iter() {
                        *selector(db.get_mut(&name).unwrap()) = Remote::Success(item);
                    }
                }

                // Failure
                Err(err) => {
                    // First publish the actual error message...
                    add_err_msg(err.to_string());

                    // ...then set the per-name entry for everything that
                    // was waiting on this to the error state.
                    for name in saved_names.into_iter() {
                        db.get_mut(&name).unwrap().gender = Remote::Error;
                    }
                }
            }
        }
    });
}
