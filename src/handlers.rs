use crate::config::Config;
use crate::db::DbRef;
use std::convert::Infallible;
use warp::http::Response;

const HELP: &str = "       _                  __
      (_)___  ____ ______/ /____
     / / __ \\/ __ `/ ___/ __/ _ \\
    / / /_/ / /_/ (__  ) /_/  __/
 __/ / .___/\\__,_/____/\\__/\\___/
/___/_/

USAGE
  $ echo hi | curl -F 'j=<-' {}
  {}/ZnD9BBwj
  $ curl {}/ZnD9BBwj
  hi 
";

#[cfg(test)]
mod test_handlers {
    use crate::handlers::HELP;
    use crate::*;

    async fn create_routes(
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let config = config::Config::init();
        let db = db::InMemoryDb::init();
        routes(config, db).await
    }

    #[tokio::test]
    async fn rejects_without_j() {
        let routes = create_routes().await;
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
    async fn rejects_non_existing_ids() {
        let routes = create_routes().await;
        let res = warp::test::request()
            .method("GET")
            .path("/doesnt-exist")
            .reply(&routes)
            .await;
        assert_eq!(res.status(), 404, "Non-existing GETs should 404");
        assert_eq!(res.body(), "", "Body for non-existing GETs should be empty");
    }

    async fn insert_and_get(msg: &[u8]) {
        let routes = create_routes().await;
        let boundary = "--boundary--";
        let body_start = format!(
            "\
         --{}\r\n\
         content-disposition: form-data; name=\"j\"\r\n\r\n",
            boundary
        );
        let body_end = format!(
            "\r\n\
         --{}--\r\n",
            boundary
        );
        let mut body = body_start.as_bytes().to_owned();
        body.extend_from_slice(msg);
        body.extend_from_slice(body_end.as_bytes());

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

        let path = url.strip_prefix("https://127.0.0.1").unwrap().trim();
        let res = warp::test::request()
            .method("GET")
            .path(path)
            .reply(&routes)
            .await;
        assert_eq!(res.status(), 200, "Valid GET request");
        assert_eq!(
            res.body(),
            msg,
            "GET request should return previously inserted content"
        );
    }

    #[tokio::test]
    async fn insert_and_get_string_content_again() {
        insert_and_get(b"my content").await;
    }

    #[tokio::test]
    async fn insert_and_get_byte_content_again() {
        insert_and_get(&[253, 254, 255]).await;
    }

    #[tokio::test]
    async fn test_help() {
        let routes = create_routes().await;
        let res = warp::test::request()
            .method("GET")
            .path("/")
            .reply(&routes)
            .await;
        assert_eq!(res.status(), 200, "GET to root should return help");
        assert_eq!(
            res.body(),
            HELP.replace("{}", "http://127.0.0.1").as_str(),
            "Should return help text"
        );
    }
}

pub async fn handle_post(
    data: Vec<u8>,
    config: Config,
    db: DbRef,
) -> Result<impl warp::Reply, Infallible> {
    if data.is_empty() {
        Ok(Response::builder().status(400).body("".to_string()))
    } else {
        let mut db = db.lock().await;
        let id = db.set(data).await;
        Ok(Response::builder()
            .status(200)
            .body(format!("{}/{}\n", config.jpaste_public_url, id)))
    }
}

pub async fn handle_get(id: String, db: DbRef) -> Result<impl warp::Reply, Infallible> {
    let mut db = db.lock().await;
    let id_future = db.get(&id);
    match id_future.await {
        Some(content) => Ok(Response::builder().status(200).body(content)),
        None => Ok(Response::builder().status(404).body(Vec::new())),
    }
}

pub async fn handle_help(config: Config) -> Result<impl warp::Reply, Infallible> {
    Ok(Response::builder()
        .status(200)
        .body(HELP.replace("{}", &config.jpaste_public_url)))
}
