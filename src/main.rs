#![warn(clippy::all)]
use crate::db::{DbRef, RedisDb};
use crate::handlers::{handle_get, handle_help, handle_post};
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

async fn routes(
    db: impl db::Db + Send + 'static,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    // let db: DbRef = Arc::new(Mutex::new(InMemoryDb::init()));
    let db: DbRef = Arc::new(Mutex::new(db));

    let post = filters::post_filter()
        .and(with_db(db.clone()))
        .and_then(handle_post);
    let get_help = filters::help_filter().and_then(handle_help);
    let get = filters::get_filter()
        .and(with_db(db.clone()))
        .and_then(handle_get);

    post.or(get).or(get_help)
}

#[tokio::main]
async fn main() {
    let localhost = [127, 0, 0, 1];
    let port = 3030;
    let addr = (localhost, port);
    let db = RedisDb::init().await;

    warp::serve(routes(db).await).run(addr).await;
}
