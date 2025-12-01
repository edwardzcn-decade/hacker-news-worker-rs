use std::fmt::Debug;

use axum::Error;
use serde::Serialize;
use worker::*;

use crate::api::HackerNewsItem;
use crate::config::{self, LIMIT_DEFAULT};
use crate::{api::hn::fetch_top, kvm::KVManager};
use axum::{http::StatusCode, response::IntoResponse, Json};

pub async fn test_job_handler() -> impl IntoResponse {
    console_log!("[Job TG] Fetch top stories without shards with Hacker News API");
    let top_items = fetch_top(Some(LIMIT_DEFAULT)).await;
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

pub async fn run_telegram_job(env: Env, shards: Option<u16>) -> Result<()> {
    // TODO
    console_log!("[Job TG] Fetch top stories without shards with Hacker News API");
    let kv: KvStore = env.kv("HACKER_NEWS_WORKER")?;
    let hn_prefix = "HN-";
    let ttl_key = config::KV_TTL_KEY;
    let ttl_val = config::KV_TTL_VAL;

    console_log!("Now get key:{}, value{:?}", "TTL_TEST", "!");
    let kvm = KVManager::init(kv, hn_prefix, ttl_key, ttl_val).await?;
    let res = kvm.get_text("TTL_TEST").await?;
    Ok(())
}
