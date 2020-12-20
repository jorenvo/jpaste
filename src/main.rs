#![warn(clippy::all)]
use crate::db::{DbRef, InMemoryDb};
use crate::handlers::{handle_get, handle_post};
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::Filter;

mod db;
mod filters;
mod handlers;
mod utils;

fn with_db(db: DbRef) -> impl Filter<Extract = (DbRef,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

fn routes() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let db: DbRef = Arc::new(Mutex::new(InMemoryDb::init()));

    let post = filters::post_filter()
        .and(with_db(db.clone()))
        .and_then(handle_post);
    let get = filters::get_filter()
        .and(with_db(db.clone()))
        .and_then(handle_get);

    post.or(get)
}

#[tokio::main]
async fn main() {
    let localhost = [127, 0, 0, 1];
    let port = 3030;
    let addr = (localhost, port);

    warp::serve(routes()).run(addr).await;
}
