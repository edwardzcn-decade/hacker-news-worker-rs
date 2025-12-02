use std::fmt::Write;
use worker::*;

use crate::api::hn::HackerNewsItem;
use crate::config::{self, LIMIT_DEFAULT};
use crate::{api::hn::fetch_top_items, kvm::KVManager};
use axum::{http::StatusCode, response::IntoResponse, Json};

pub async fn test_job_handler() -> impl IntoResponse {
    console_log!("[Job TG] Fetch top stories without shards with Hacker News API");
    let top_items = fetch_top_items(Some(LIMIT_DEFAULT)).await;
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

// TODO add shards
pub async fn run_telegram_job(env: Env, _shards: Option<u16>) -> Result<()> {
    // TODO
    console_log!("[Job TG] Fetch top stories without shards with Hacker News API");
    let kv: KvStore = env.kv("HACKER_NEWS_WORKER_RS")?;
    let hn_prefix = "HN-";
    let ttl_key = config::KV_TTL_KEY;
    let ttl_val = config::KV_TTL_VAL;

    let kvm = KVManager::init(kv, hn_prefix, ttl_key, ttl_val).await?;
    // TODO may need design api error
    let top_items = fetch_top_items(None)
        .await
        .map_err(|e| worker::Error::RustError(e.to_string()))?;
    let cached_ids = kvm
        .list_keys(Some(hn_prefix), true)
        .await?
        .iter()
        .map(|id| {
            id.strip_prefix(hn_prefix)
                .unwrap_or(id.as_str())
                .parse::<u64>()
                // Change ParseInterror
                .map_err(|e| worker::Error::RustError(e.to_string()))
        })
        .collect::<Result<Vec<u64>>>()?;
    console_log!(
        "[Job TG] Cached Hacker News itme ids (parse to to u64):{:?}",
        cached_ids
    );

    let filtered_items = top_items
        .into_iter()
        .filter(|item| {
            item.score.unwrap_or(0) >= config::MIN_SCORE_DEFAULT
                && item.timestamp >= config::UNIX_TIME_DEFAULT
                && !cached_ids.contains(&item.item_id)
        })
        .collect::<Vec<HackerNewsItem>>();
    console_log!(
        "[Job TG] Filter items, show ids (map to u64):{:?}",
        filtered_items
            .iter()
            .map(|i| i.item_id)
            .collect::<Vec<u64>>()
    );
    for item in &filtered_items {
        // TODO no parallel
        // TODO make prefix factory
        let kk = format!("{}{}", hn_prefix, item.item_id);
        let vv = stringify!(item);
        console_log!("[Job TG] Try cache key:{kk} value:{vv} with test meta and default ttl",);
        // TODO change this
        kvm.create(kk, vv, Some(vv), None).await?;
    }

    notify_all(env, filtered_items, None).await?;
    console_log!("[Jpb TG] After notify all");
    Ok(())
}

async fn notify_all(
    env: Env,
    payloads: Vec<HackerNewsItem>,
    specified_bots: Option<Vec<String>>,
) -> Result<()> {
    if let None = specified_bots {
        console_warn!(
            "[Notify All] ⚠️ notifyTg with specified bot not implement. Fallback to default bot."
        );
    }
    let tg_token = env.secret("TG_BOT_TOKEN")?.to_string();
    let tg_chat_id = env.var("TG_CHAT_ID")?.to_string();
    for p in payloads {
        console_log!(
            "[Notify All] Title: \"{:?}\" --- By: {}\n[Notify All] Link: {:?}",
            &p.title,
            &p.by,
            &p.url
        );
        notify_tg(&tg_token, &tg_chat_id, &p, None).await?
    }
    Ok(())
}

async fn notify_tg(
    tg_token: &str,
    tg_chat_id: &str,
    payload: &HackerNewsItem,
    specified_bot: Option<String>,
) -> Result<()> {
    if let None = specified_bot {
        console_warn!(
            "[Notify TG] ⚠️ notifyTg with specified bot not implement. Fallback to default bot."
        );
    }
    let story_id = payload.item_id.to_string();
    // TODO finish the base 65 encode
    let short_id = story_id.clone();

    let cc_option = payload.decendants;
    // Comment url group
    let hn_url =
        Url::parse_with_params("https://news.ycombinator.com/item/", &[("id", &story_id)])?;
    let short_hn_url = Url::parse(format!("https://readhacker.news/c/{}", short_id).as_str())?;
    // Story url group
    let story_url = match payload.url.as_deref() {
        Some(s) => Url::parse(s)?,
        None => {
            // TODO need clone?
            hn_url
        }
    };
    let short_story_url = match payload.url.as_deref() {
        Some(_) => Url::parse(format!("https://readhacker.news/s/{}", short_id).as_str())?,
        None => {
            // TODO need clone?
            short_hn_url.clone()
        }
    };
    // Build buttons
    let buttons = serde_json::json!([
        {
            "text": payload.url.as_deref().map_or_else(
                || "Read",
                |_| "Read HN",
            ),
            "url": story_url.clone(),
        },
        {
            "text": cc_option.map_or_else(
                || "Comments".to_string(),
                |cc| format!("Comments {}+", cc),
            ),
            "url": short_hn_url.clone(),
        },
    ]);
    let reply_markup = serde_json::json!({
        "inline_keyboard": [buttons],
    });

    // TODO Build 🔥
    // Build message
    let msg = build_tg_message(
        payload,
        "🦀 ",
        short_story_url.as_str(),
        short_hn_url.as_str(),
    );
    let res = crate::api::tg::send_message(tg_token, tg_chat_id, &msg, reply_markup).await?;
    if !(200..300).contains(&res.status_code()) {
        console_warn!(
            "[Notify TG] ❌ notifyTg fails. Code: {}.",
            &res.status_code()
        );
        return Err(worker::Error::RustError("failed to fetch max item".into()));
    }
    Ok(())
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn build_tg_message(
    payload: &HackerNewsItem,
    status_emoji: &str,
    short_story_url: &str,
    short_hn_url: &str,
) -> String {
    let mut msg = String::new();
    // Add title
    let title = escape_html(payload.title.as_deref().unwrap_or("Untitled"));

    // Add Score
    let score_part = payload
        .score
        .map(|s| format!("Score: {}+", s))
        .unwrap_or_default();
    let by_part = format!("by {}", payload.by);

    write!(&mut msg, "<b>{}</b> ${}", title, status_emoji);
    if score_part.is_empty() {
        write!(&mut msg, "\n({})", by_part);
    } else {
        write!(&mut msg, "\n({} · {})", score_part, by_part);
    }
    // Add Story and Comments Link
    write!(
        &mut msg,
        "\n\n<b>Link:</b> {}\n<b>Comments:</b> {}",
        short_story_url, short_hn_url
    );
    msg
}
