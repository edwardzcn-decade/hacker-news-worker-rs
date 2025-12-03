use axum::{
    extract::Path,
    http::{StatusCode, Uri},
    response::{IntoResponse, Redirect},
};
use worker::{console_log, console_warn};

use crate::api::hn::LiveDataKey;

pub async fn get_root() -> &'static str {
    console_log!("[Router] Trigger get_root");
    "Hello Axum!"
}

pub async fn get_about() -> impl IntoResponse {
    console_log!("[Router] Trigger get_about");
    (StatusCode::OK, "Hey this is about page into response").into_response()
}

pub async fn get_blog() -> impl IntoResponse {
    console_log!("[Router] Trigger get_blog");
    // TODO, change Redirect to calling hacker news api
    Redirect::to("https://edwardzcn.me").into_response()
}

pub async fn get_forward_key(Path(key): Path<LiveDataKey>) -> impl IntoResponse {
    console_log!("[Router] Trigger post_forward_key");
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
    console_log!("[Router] Trigger post_forward_item");
    if item != "item" {
        let msg = "Only forward/item/<number> is allowed";
        console_warn!("[Router] ⚠️ {}", msg);
        return (StatusCode::BAD_REQUEST, msg).into_response();
    }
    // TODO, change Redirect to calling hacker news api
    Redirect::to(format!("https://hacker-news.firebaseio.com/v0/{}/{}.json", item, id).as_str())
        .into_response()
}

pub async fn fallback_handler(uri: Uri) -> impl IntoResponse {
    console_log!("[Router] Trigger fallback_handler");
    (StatusCode::NOT_FOUND, format!("404 Not Found: {}", uri))
}
