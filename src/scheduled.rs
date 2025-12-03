use std::fmt::Write;
use worker::*;

use crate::api::hn::HackerNewsItem;
use crate::config::{KV_TTL_KEY, KV_TTL_VAL, MIN_SCORE_DEFAULT, UNIX_TIME_DEFAULT};
use crate::{
    api::hn::fetch_top_items,
    kvm::{KVManager, KVMeta},
};

// TODO add shards
pub async fn run_telegram_job(env: Env, _shards: Option<u16>) -> Result<()> {
    console_log!("[Job TG] Fetch top stories without shards with Hacker News API");
    let hn_prefix = "HN-";
    let ttl_key = KV_TTL_KEY;
    let ttl_val = KV_TTL_VAL;
    let kv: KvStore = env.kv("HACKER_NEWS_WORKER_RS")?;
    let kvm = KVManager::init(kv, hn_prefix, ttl_key, ttl_val).await?;
    // TODO may need design api error
    let top_items = fetch_top_items(None)
        .await
        .map_err(|e| Error::RustError(e.to_string()))?;

    // Note: No test for listKeys with setting `onlyOnce` false
    let raw_cached_ids = kvm.list_keys(Some(hn_prefix), true).await?;
    let cached_ids = raw_cached_ids
        .into_iter()
        .filter_map(|prefixed_id| {
            let stipped = match prefixed_id.strip_prefix(hn_prefix) {
                Some(i) => i,
                None => {
                    console_warn!("[Job TG] ⚠️ Skip unexpected KV key:{}", &prefixed_id);
                    return None;
                }
            };
            // parse to u64 and discard the error
            stipped.parse::<u64>().ok()
        })
        .collect::<Vec<u64>>();
    console_log!(
        "[Job TG] Cached Hacker News itme ids (parse to to u64):{:?}",
        cached_ids
    );

    let filtered_items = top_items
        .into_iter()
        .filter(|item| {
            item.score.unwrap_or(0) >= MIN_SCORE_DEFAULT
                && item.timestamp >= UNIX_TIME_DEFAULT
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
        let vv = serde_json::to_string(item)?;
        console_log!(
            "[Job TG] Try cache id:{} with metadata... and ttl(default).",
            item.item_id
        );
        let uuid = uuid::Uuid::new_v4();
        let mut metas = KVMeta::new(uuid);
        metas
            .with_llm_summary(Some("Test".to_string()))
            .with_llm_score(Some("Test".to_string()));
        kvm.create(kk, vv, Some(metas), None).await?;
    }

    notify_all(env, filtered_items, None).await?;
    Ok(())
}

async fn notify_all(
    env: Env,
    payloads: Vec<HackerNewsItem>,
    specified_bots: Option<Vec<String>>,
) -> Result<()> {
    if let None = specified_bots {
        console_warn!(
            "[Notify] ⚠️ notifyAll with specifiedBots (bot list) not implement. Fallback to default bot."
        );
    }
    let tg_token = env
        .secret("TG_BOT_TOKEN")
        .map_err(|e| {
            console_error!(
                "[Notify] ❌ Error in notifyTg, Telegram bot token missing in Env. Please Check."
            );
            e
        })?
        .to_string();
    let tg_chat_id = env.var("TG_CHAT_ID")
        .map_err(|e| {
            console_error!("[Notify] ❌ Error in notifyTg, Telegram Chat ID (may use \'@xxx\') missing in Env. Please Check.");
            e
        })?
        .to_string();
    for p in payloads {
        console_log!(
            "[Notify] Title: \"{}\" --- By: {}\n[Notify] Link: {}",
            &p.title.as_deref().unwrap_or_default(),
            &p.by,
            &p.url.as_deref().unwrap_or_default()
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
            "[Notify] ⚠️ notifyTg with specified bot not implement. Fallback to default bot."
        );
    }
    let story_id = payload.item_id.to_string();
    let short_id = bs58::encode(&story_id.as_bytes()).into_string();

    let cc_option = payload.decendants;
    // Comment url group
    let hn_url: String =
        Url::parse_with_params("https://news.ycombinator.com/item/", &[("id", &story_id)])?
            .to_string();
    let short_hn_url: String = format!("https://readhacker.news/c/{}", &short_id);
    // Story url group
    let story_url: String = payload.url.as_deref().unwrap_or(&hn_url).to_string();
    let short_story_url: String = payload
        .url
        .as_deref()
        .map(|_| format!("https://readhacker.news/s/{}", &short_id))
        .unwrap_or(short_hn_url.clone());
    // Build buttons
    let buttons = serde_json::json!([
        {
            "text": payload.url.as_deref().map_or_else(
                || "Read",
                |_| "Read HN",
            ),
            "url": story_url,
        },
        {
            "text": cc_option.map_or_else(
                || "Comments".to_string(),
                |cc| format!("Comments {}+", cc),
            ),
            "url": short_hn_url,
        },
    ]);
    let reply_markup = serde_json::json!({
        "inline_keyboard": [buttons],
    });

    // TODO Build 🔥 or ❄️
    // Build message
    let msg = build_tg_message(payload, "🦀 ", &short_story_url, &short_hn_url);
    let res = crate::api::tg::send_message(tg_token, tg_chat_id, &msg, reply_markup).await?;
    if !(200..300).contains(&res.status_code()) {
        console_error!("[Notify] ❌ notifyTg fails. Code: {}.", &res.status_code());
        return Err(Error::RustError("failed to fetch max item".into()));
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

    write!(&mut msg, "<b>{}</b> {}", title, status_emoji);
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
