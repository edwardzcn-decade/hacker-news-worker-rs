pub const APP_NAME: &str = "hacker-news-worker-rs";
pub const APP_VERSION: &str = "0.1.0";
pub const APP_USER_AGENT: &str = "Cloudflare Worker - hacker-news-worker-rs/0.1.0";

pub const LIMIT_DEFAULT: u16 = 20;
pub const KV_TTL_KEY: &str = "TTL";
pub const KV_TTL_VAL: u64 = 86400;
pub const MIN_SCORE_DEFAULT: u64 = 150;
pub const UNIX_TIME_DEFAULT: u64 = 0;
