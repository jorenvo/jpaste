#![warn(clippy::all)]
use crate::db::{Db, NaiveDb};
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::http::Response;
use warp::Filter;

mod db;
mod filters;
mod utils;

type DbRef = Arc<Mutex<dyn Db + Send>>;

#[cfg(test)]
mod test_handlers {
    use crate::*;

    #[tokio::test]
    async fn rejects_without_j() {
        let routes = routes();
        let res = warp::test::request()
            .method("POST")
            .path("/")
            .header("content-type", "multipart/form-data; boundary=yolo")
            .body("random content")
            .reply(&routes)
            .await;
        assert_eq!(res.status(), 400, "Should include j= in body");
        assert_eq!(res.body(), "", "Don't be too verbose ;)");
    }

    #[tokio::test]
    async fn accepts_with_j() {
        let filter = routes();
        let boundary = "--boundary--";
        let body = format!(
            "\
         --{0}\r\n\
         content-disposition: form-data; name=\"j\"\r\n\r\n\
         my value\r\n\
         --{0}--\r\n\
         ",
            boundary
        );
        let res = warp::test::request()
            .method("POST")
            .path("/")
            .header(
                "content-type",
                format!("multipart/form-data; boundary={}", boundary),
            )
            .body(body)
            .reply(&filter)
            .await;

        assert_eq!(res.status(), 200, "Valid request");
        assert!(
            res.body().starts_with(b"https://127.0.0.1/"),
            "Should return URL to content"
        );
    }
}

fn with_db(db: DbRef) -> impl Filter<Extract = (DbRef,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

async fn handle_post(data: String, db: DbRef) -> Result<impl warp::Reply, Infallible> {
    println!("got data: {}", &data);
    if data.is_empty() {
        Ok(Response::builder().status(400).body("".to_string()))
    } else {
        let mut db = db.lock().await;
        let id = db.set_data(data).await;
        Ok(Response::builder()
            .status(200)
            .body(format!("https://127.0.0.1/{}", id)))
    }
}

fn routes() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let db: DbRef = Arc::new(Mutex::new(NaiveDb::init()));
    // let x = with_db(db);
    filters::post_filter()
        .and(with_db(db))
        .and_then(handle_post)
}

#[tokio::main]
async fn main() {
    let localhost = [127, 0, 0, 1];
    let port = 3030;
    let addr = (localhost, port);

    warp::serve(routes()).run(addr).await;
}
