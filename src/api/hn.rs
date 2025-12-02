use crate::config::{APP_USER_AGENT, LIMIT_DEFAULT};
use serde::{Deserialize, Serialize};
use worker::{
    console_error, console_log, console_warn, Error, Fetch, Method, Request, Response, Url,
};

const HN_BASE_URL: &str = "https://hacker-news.firebaseio.com/v0/";
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HackerNewsItem {
    #[serde(rename = "id", alias = "item_id")]
    pub item_id: u64,
    #[serde(rename = "deleted", alias = "item_deleted")]
    item_deleted: Option<bool>,
    #[serde(rename = "dead", alias = "item_dead")]
    item_dead: Option<bool>,
    #[serde(rename = "type", alias = "item_type")]
    item_type: Option<String>,
    pub by: String,
    #[serde(rename = "time", alias = "timestamp")]
    pub timestamp: u64,
    pub text: Option<String>,
    parent: Option<u64>,
    poll: Option<u64>,
    kids: Option<Vec<u64>>,
    pub url: Option<String>,
    pub score: Option<u64>,
    pub title: Option<String>,
    parts: Option<Vec<u64>>,
    pub decendants: Option<u64>,
}

impl HackerNewsItem {
    pub fn mock() -> Self {
        Self {
            item_id: 1,
            item_deleted: Some(false),
            item_dead: Some(false),
            item_type: Some("story".into()),
            by: "tester".into(),
            timestamp: 1_700_000_000,
            text: Some("Test text".into()),
            parent: None,
            poll: None,
            kids: Some(vec![2, 3]),
            url: Some("https://example.com".into()),
            score: Some(200),
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

impl std::str::FromStr for LiveDataKey {
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

// Get top stories with no shards
pub async fn fetch_top_items(limit: Option<u16>) -> Result<Vec<HackerNewsItem>, Error> {
    let limit = limit.unwrap_or(LIMIT_DEFAULT);
    let ids = fetch_top_stories(Some(limit)).await?;
    console_warn!("In fetch_top ids:{:?}", ids);
    let items = fetch_items(&ids).await?;
    Ok(items)
}

async fn fetch_json_response(base: &str, endpoint: &str) -> Result<Response, Error> {
    let url = Url::parse_with_params(
        format!("{}{}", base, endpoint).as_str(),
        &[("print", "pretty")],
    )?;
    console_warn!("In fetch_json_response url:{:?}", url);
    let mut req = Request::new(url.as_str(), Method::Get)?;
    {
        let headers = req.headers_mut()?;
        headers.set("User-Agent", APP_USER_AGENT)?;
        headers.set("Accept", "application/json")?;
    }
    let res = Fetch::Request(req).send().await?;
    Ok(res)
}

pub async fn fetch_max_item() -> Result<u64, Error> {
    let endpoint = "maxitem.json";
    let mut res = fetch_json_response(HN_BASE_URL, endpoint).await?;
    if !(200..300).contains(&res.status_code()) {
        console_error!("In fetch_max_item. Failed to fetch max item id");
        return Err(Error::RustError("failed to fetch max item".into()));
    }
    let m = res.json::<u64>().await?;
    Ok(m)
}

pub async fn fetch_top_stories(limit: Option<u16>) -> Result<Vec<u64>, Error> {
    // TODO add limit
    let endpoint = "topstories.json";
    let mut res = fetch_json_response(HN_BASE_URL, endpoint).await?;
    if !(200..300).contains(&res.status_code()) {
        console_error!(
            "In fetch_top_stories. Failed to fetch top stories with status:{:?}",
            &res.status_code()
        );
        return Err(Error::RustError("failed to fetch top stories".into()));
    }
    let mut v = res.json::<Vec<u64>>().await?;
    // FIXME hard code limit_default
    v.truncate(limit.unwrap_or(20).into());
    Ok(v)
    // Only test
    // Ok(vec![46103532, 46101492, 46100323, 46106556, 46106132])
}

pub async fn fetch_items(ids: &[u64]) -> Result<Vec<HackerNewsItem>, Error> {
    let mut items = Vec::with_capacity(ids.len());
    for &id in ids {
        let item = fetch_item(id).await?;
        console_log!("Get item:{:?}", item);
        items.push(item);
    }
    Ok(items)
}

pub async fn fetch_item(id: u64) -> Result<HackerNewsItem, Error> {
    let endpoint = format!("item/{}.json", id);
    let mut res = fetch_json_response(HN_BASE_URL, &endpoint).await?;
    if !(200..300).contains(&res.status_code()) {
        console_error!(
            "In fetch_item. Failed to fetch single item id:{} with status:{}",
            id,
            &res.status_code()
        );
        return Err(Error::RustError(
            format!("failed to fetch single item id:{}", id).into(),
        ));
    }
    let hn_item = res.json::<HackerNewsItem>().await?;
    Ok(hn_item)
    // Only for test
    // Ok(HackerNewsItem::mock())
}
