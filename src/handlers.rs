use crate::db::DbRef;
use std::convert::Infallible;
use warp::http::Response;

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
    async fn insert_and_get_content_again() {
        let routes = routes();
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
            .reply(&routes)
            .await;

        assert_eq!(res.status(), 200, "Valid request");

        let url = std::str::from_utf8(res.body()).unwrap();
        assert!(
            url.starts_with("https://127.0.0.1/"),
            "Should return URL to content"
        );

        let path = url.strip_prefix("https://127.0.0.1").unwrap();
        let res = warp::test::request()
            .method("GET")
            .path(path)
            .reply(&routes)
            .await;
        assert_eq!(res.status(), 200, "Valid GET request");
    }
}

pub async fn handle_post(data: String, db: DbRef) -> Result<impl warp::Reply, Infallible> {
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

pub async fn handle_get(_id: String, _db: DbRef) -> Result<impl warp::Reply, Infallible> {
    Ok(Response::builder().status(200).body("".to_string()))
}
