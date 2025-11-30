pub mod api;
pub mod config;

use axum::{
    extract::Path,
    http::{StatusCode, Uri},
    response::{IntoResponse, Redirect},
    routing::get,
    Json, Router,
};
use std::str::FromStr;
use tower_service::Service;

use api::hn::LiveDataKey;
use config::LIMIT_DEFAULT;

use tracing::{error, trace};
use worker::*;

impl LiveDataKey {
    pub fn as_str(&self) -> &'static str {
        match self {
            LiveDataKey::MaxItem => "max_item",
            LiveDataKey::TopHn => "top_hn",
            LiveDataKey::NewHn => "new_hn",
            LiveDataKey::BestHn => "best_hn",
            LiveDataKey::AskHn => "ask_hn",
            LiveDataKey::ShowHn => "show_hn",
            LiveDataKey::JobHn => "job_hn",
            LiveDataKey::Updates => "updates",
        }
    }
}

impl FromStr for LiveDataKey {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "max_item" => Ok(LiveDataKey::MaxItem),
            "top_hn" => Ok(LiveDataKey::TopHn),
            "new_hn" => Ok(LiveDataKey::NewHn),
            "best_hn" => Ok(LiveDataKey::BestHn),
            "ask_hn" => Ok(LiveDataKey::AskHn),
            "show_hn" => Ok(LiveDataKey::ShowHn),
            "job_hn" => Ok(LiveDataKey::JobHn),
            "updates" => Ok(LiveDataKey::Updates),
            _ => Err(format!("Unknown LiveDataKey: {}", s)),
        }
    }
}

fn router() -> Router {
    Router::new()
        .route("/", get(get_root))
        .route("/about", get(get_about))
        .route("/blog", get(get_blog))
        .route("/forward/{key}", get(get_forward_key))
        .route("/forward/{item}/{id}", get(get_forward_item))
        .route("/test", get(test_job_handler))
        .fallback(fallback_handler)
    // .with_state(state)
}

#[event(fetch)]
async fn fetch(
    req: HttpRequest,
    _env: Env,
    _ctx: Context,
) -> Result<axum::http::Response<axum::body::Body>> {
    Ok(router().call(req).await?)
}

pub async fn get_root() -> &'static str {
    console_log!("Trigger get_root");
    "Hello Axum!"
}

pub async fn get_about() -> impl IntoResponse {
    console_log!("Trigger get_about");
    (StatusCode::OK, "Hey this is about page into response").into_response()
}

pub async fn get_blog() -> impl IntoResponse {
    console_log!("Trigger get_blog");
    Redirect::to("https://edwardzcn.me").into_response()
}

pub async fn get_forward_key(Path(key): Path<LiveDataKey>) -> impl IntoResponse {
    console_log!("Trigger post_forward_key");
    let k = match key {
        LiveDataKey::MaxItem => "maxitem",
        LiveDataKey::TopHn => "topstories",
        LiveDataKey::NewHn => "newstories",
        LiveDataKey::BestHn => "beststories",
        LiveDataKey::AskHn => "askstories",
        LiveDataKey::ShowHn => "showstories",
        LiveDataKey::JobHn => "jobstories",
        LiveDataKey::Updates => "updates",
    };
    // TODO, change Redirect to calling hacker news api
    Redirect::to(format!("https://hacker-news.firebaseio.com/v0/{}.json", k).as_str())
        .into_response()
}

pub async fn get_forward_item(Path((item, id)): Path<(String, u64)>) -> impl IntoResponse {
    console_log!("Trigger post_forward_item");
    if item != "item" {
        let msg = "Only forward/item/<number> is allowed";
        console_error!("{}", msg);
        return (StatusCode::BAD_REQUEST, msg).into_response();
    }
    // item id url
    // TODO, change Redirect to calling hacker news api
    Redirect::to(format!("https://hacker-news.firebaseio.com/v0/{}/{}.json", item, id).as_str())
        .into_response()
}

pub async fn test_job_handler() -> impl IntoResponse {
    console_log!("[Job TG] Fetch top stories without shards with Hacker News API");
    let top_items = api::hn::fetch_top(Some(LIMIT_DEFAULT)).await;
    match top_items {
        Ok(v) => {
            console_warn!("Test");
            (StatusCode::OK, Json(v)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Saaaaad Test {}", e.to_string()),
        )
            .into_response(),
    }
}

pub async fn fallback_handler(uri: Uri) -> impl IntoResponse {
    console_log!("Trigger fallback_handler");
    (StatusCode::NOT_FOUND, format!("404 Not Found: {}", uri))
}
