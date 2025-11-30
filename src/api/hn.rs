use std::vec;

use crate::config::LIMIT_DEFAULT;
use axum::Error;
use serde::{Deserialize, Serialize};

const HN_BASE_URL: &str = "https://hacker-news.firebaseio.com/v0/";
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HackerNewsItem {
    id: u64,
    is_deleted: Option<bool>,
    is_dead: Option<bool>,
    item_type: Option<String>,
    by: String,
    timestamp: u64,
    text: Option<String>,
    parent: Option<u64>,
    poll: Option<u64>,
    kids: Option<Vec<u64>>,
    url: Option<String>,
    score: Option<u64>,
    title: Option<String>,
    parts: Option<Vec<u64>>,
    decendants: Option<u64>,
}

impl HackerNewsItem {
    pub fn mock() -> Self {
        Self {
            id: 1,
            is_deleted: Some(false),
            is_dead: Some(false),
            item_type: Some("story".into()),
            by: "tester".into(),
            timestamp: 1_700_000_000,
            text: Some("Test text".into()),
            parent: None,
            poll: None,
            kids: Some(vec![2, 3]),
            url: Some("https://example.com".into()),
            score: Some(123),
            title: Some("Test Title".into()),
            parts: None,
            decendants: Some(10),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LiveDataTypes {
    MaxItem,
    TopHn,
    NewHn,
    BestHn,
    AskHn,
    ShowHn,
    JobHn,
    Updates,
}
pub type LiveDataKey = LiveDataTypes;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LiveDataConfig {
    endpoint: String,
    label: String,
    description: Option<String>,
    default_limit: Option<u16>,
    default_score: Option<u16>,
}

pub type LiveDataValue = LiveDataConfig;

// Fetch top stories with no shards
pub async fn fetch_top(limit: Option<u16>) -> Result<Vec<HackerNewsItem>, Error> {
    let limit = limit.unwrap_or(LIMIT_DEFAULT);
    let ids = fetch_top_stories(Some(limit)).await?;
    worker::console_warn!("In fetch_top ids:{:?}", ids);
    let items = fetch_items(&ids).await?;
    Ok(items)
}

pub async fn fetch_top_stories(limit: Option<u16>) -> Result<Vec<u64>, Error> {
    Ok(vec![1, 2, 3, 4, 5])
}

pub async fn fetch_items(ids: &[u64]) -> Result<Vec<HackerNewsItem>, Error> {
    let mut items = Vec::with_capacity(ids.len());
    for &id in ids {
        let item = fetch_item(id).await?;
        items.push(item);
    }
    Ok(items)
}

pub async fn fetch_item(id: u64) -> Result<HackerNewsItem, Error> {
    Ok(HackerNewsItem::mock())
}
