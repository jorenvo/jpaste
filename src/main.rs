use std::collections::HashMap;
use std::convert::Infallible;
use warp::http::Response;
use warp::Filter;
use warp::Rejection;

const MAX_PAYLOAD: u64 = 1024 * 1024; // 1 MB

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
    }

    #[tokio::test]
    async fn accepts_valid() {
        let filter = post_filter();
        let res = warp::test::request()
            .method("POST")
            .path("/")
            .body("random content")
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
        let filter = routes();
        let res = warp::test::request()
            .method("POST")
            .path("/")
            .body("random content")
            .reply(&filter)
            .await;
        assert_eq!(res.status(), 400, "Should include j= in body");
        assert_eq!(res.body(), "", "Don't be too verbose ;)")
    }

    #[tokio::test]
    async fn accepts_with_j() {
        let filter = routes();
        let res = warp::test::request()
            .method("POST")
            .path("/")
            .body("j=my_content")
            .reply(&filter)
            .await;
        assert_eq!(res.status(), 200, "Valid request");
        assert!(
            res.body().starts_with(b"https://127.0.0.1/"),
            "Should return URL to content"
        );
    }
}

fn post_filter() -> impl Filter<Extract = (String,), Error = Rejection> + Clone {
    warp::post()
        .and(warp::path::end()) // match only /
        .and(warp::body::content_length_limit(MAX_PAYLOAD))
        .and(warp::body::form())
        .map(|mut form_map: HashMap<String, String>| form_map.remove("j").unwrap_or(String::new()))
}

async fn handle_post(data: String) -> Result<impl warp::Reply, Infallible> {
    if data.is_empty() {
        Ok(Response::builder().status(400).body("".to_string()))
    } else {
        Ok(Response::builder()
            .status(200)
            .body(format!("https://127.0.0.1/")))
    }
    //     // Ok(StatusCode::BAD_REQUEST)
    // } else {
    //     // Ok("some_url".to_string())
    // }
    // Ok("abc".to_string())
}

fn routes() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    post_filter().and_then(handle_post)
}

#[tokio::main]
async fn main() {
    // let hi = warp::path("hello")
    //     .and(warp::path::param())
    //     .and(warp::header("user-agent"))
    //     .map(|param: String, agent: String| format!("Hello {}, whose agent is {}", param, agent));

    let localhost = [127, 0, 0, 1];
    let port = 3030;
    let addr = (localhost, port);

    warp::serve(routes()).run(addr).await;
}
