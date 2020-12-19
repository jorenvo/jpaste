#![warn(clippy::all)]
use crate::db::{Db, NaiveDb};
use std::convert::Infallible;
use std::str;
use std::sync::Arc;
use tokio::stream::StreamExt;
use tokio::sync::Mutex;
use warp::http::Response;
use warp::Buf;
use warp::Filter;
use warp::Rejection;

mod db;
mod utils;

const MAX_PAYLOAD: u64 = 1024 * 1024; // 1 MB

type DbRef = Arc<Mutex<dyn Db + Send>>;

#[cfg(test)]
mod test_filters {
    use crate::*;
    use std::convert::TryInto;

    #[tokio::test]
    async fn rejects_invalid() {
        let filter = post_filter();
        let mut res = warp::test::request()
            .method("GET")
            .path("/")
            .reply(&filter)
            .await;
        assert_eq!(res.status(), 405, "Only POSTs to / are allowed");
        res = warp::test::request()
            .method("POST")
            .path("/uea")
            .reply(&filter)
            .await;
        assert_eq!(res.status(), 404, "Only POSTs to / are allowed");

        let payload = vec![b'X'; (MAX_PAYLOAD + 1).try_into().unwrap()];
        res = warp::test::request()
            .method("POST")
            .path("/")
            .body(payload)
            .reply(&filter)
            .await;
        assert_eq!(res.status(), 413, "Payload is too large");

        let res = warp::test::request()
            .method("POST")
            .path("/")
            .body("hi")
            .reply(&filter)
            .await;
        assert_eq!(res.status(), 400, "POSTs need a multipart form body");
    }

    #[tokio::test]
    async fn accepts_valid() {
        let filter = post_filter();
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
        assert_eq!(res.status(), 200, "POSTS to / are allowed");
    }
}

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

async fn get_content(mut form_data: warp::multipart::FormData) -> Result<String, Infallible> {
    // form_data is a Stream that yields name: content. content is also a Stream.
    // TODO: can we warp reject here? I think not because it cannot be done statically.
    let next_data = form_data.next().await;
    if let Some(value) = next_data {
        if let Ok(first_part) = value {
            if first_part.name() != "j" {
                Ok(String::new())
            } else {
                let mut val: Vec<u8> = Vec::new();
                let mut data_stream = first_part.stream();
                while let Some(partial_val) = data_stream.next().await {
                    val.extend(partial_val.unwrap().bytes());
                }

                Ok(str::from_utf8(&val).unwrap().to_string())
            }
        } else {
            // body is not a valid multipart form
            Ok(String::new())
        }
    } else {
        // no form body
        Ok(String::new())
    }
}

fn post_filter() -> impl Filter<Extract = (String,), Error = Rejection> + Clone {
    warp::post()
        .and(warp::path::end()) // match only /
        .and(warp::body::content_length_limit(MAX_PAYLOAD))
        .and(warp::filters::multipart::form())
        .and_then(get_content)
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
    post_filter().and(with_db(db)).and_then(handle_post)
}

#[tokio::main]
async fn main() {
    let localhost = [127, 0, 0, 1];
    let port = 3030;
    let addr = (localhost, port);

    warp::serve(routes()).run(addr).await;
}
