use axum::{async_trait, http::{header, StatusCode}, response::Response, routing::{get, post}, Json, Router, middleware};
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use jwt_lib::User;
use serde_json::json;
use tokio::net::TcpListener;

// https://www.youtube.com/watch?v=n2M4A4mO0QU

// https://www.youtube.com/watch?v=yjeTOPR3Syg

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
  let routes = Router::new()
    .route("/public-view", get(public_view_handler))
    .route("/get-token", post(get_token_handler))
    .nest(
      "/secret-view",
      Router::new()
        .route("/", get(secret_view_handler))
        .route("/a", get(|| async { "A" }))
        .route("/abc", get(|| async { "ABC" }))
        .route_layer(middleware::from_extractor::<Auth>())
    );

  let tcp_listener = TcpListener::bind("127.0.0.1:2323")
    .await
    .expect("Address should be free and valid");

  axum::serve(tcp_listener, routes)
    .await
    .expect("Error serving application");
}

async fn public_view_handler() -> Response<String> {
  Response::builder()
    .status(StatusCode::OK)
    .header(header::CONTENT_TYPE, "application/json")
    .body(
      json!({
        "success": true,
        "data": {
          "message": "This data is available for public view"
        }
      })
      .to_string(),
    )
    .unwrap_or_default()
}

async fn get_token_handler(Json(user): Json<User>) -> Response<String> {
  let token = jwt_lib::get_jwt(user);

  match token {
    Ok(token) => Response::builder()
      .status(StatusCode::OK)
      .header(header::CONTENT_TYPE, "application/json")
      .body(
        json!({
        "success": true,
        "data": {
          "token": token
        }
      })
          .to_string(),
      )
      .unwrap_or_default(),
    Err(e) => Response::builder()
      .status(StatusCode::BAD_REQUEST)
      .header(header::CONTENT_TYPE, "application/json")
      .body(
        json!({
        "success": false,
        "data": {
          "message": e
        }
      })
          .to_string(),
      )
      .unwrap_or_default()
  }

}

async fn secret_view_handler(
  Auth(user): Auth
) -> Response<String> {
  Response::builder()
    .status(StatusCode::OK)
    .header(header::CONTENT_TYPE, "application/json")
    .body(
      json!({
        "success": true,
        "data": user
      })
      .to_string(),
    )
    .unwrap_or_default()
}

struct Auth(User);

#[async_trait]
impl <S> FromRequestParts<S> for Auth where S: Send + Sync {
  type Rejection = Response<String>;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
    let access_token = parts
      .headers
      .get(header::AUTHORIZATION)
      .and_then(|value| value.to_str().ok())
      .and_then(|str| str.split(" ").nth(1));

    match access_token {
      Some(token) => {
        let user = jwt_lib::decode_jwt(token);

        match user {
          Ok(user) => Ok(Auth(user)),
          Err(e) => Err(
            Response::builder()
              .status(StatusCode::UNAUTHORIZED)
              .header(header::CONTENT_TYPE, "application/json")
              .body(
                json!({
            "success": false,
            "data": {
              "message": e,
            }}).to_string(),
              )
              .unwrap_or_default()
          )
        }
      }

      None => Err(
        Response::builder()
          .status(StatusCode::UNAUTHORIZED)
          .header(header::CONTENT_TYPE, "application/json")
          .body(
            json!({
            "success": false,
            "data": {
              "message": "no token provided"
            }
          }).to_string(),
          )
          .unwrap_or_default()
      )
    }
  }
}