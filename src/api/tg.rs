use crate::config::APP_USER_AGENT;
use serde_json::Value;
use wasm_bindgen::JsValue;
use worker::{console_log, Error, Fetch, Method, Request, RequestInit, Response, Url};

const TG_BASE_URL: &str = "https://api.telegram.org/";

pub async fn send_message(
    token: &str,
    chat_id: &str,
    msg: &str,
    reply_markup: Value,
) -> Result<Response, Error> {
    console_log!("In send_message msg:{}", msg);
    let url = Url::parse(format!("{}bot{}/sendMessage", TG_BASE_URL, token).as_str())?;
    let payload = serde_json::json!({
      "chat_id": chat_id,
      "text": msg,
      "parse_mode": "HTML",
      "reply_markup": reply_markup,
      "disable_web_page_preview": false,
    });
    let payload_str = payload.to_string();
    let mut init = RequestInit::new();
    {
        init.with_method(Method::Post)
            .with_body(Some(JsValue::from_str(&payload_str)));
    }
    let mut req = Request::new_with_init(url.as_str(), &init)?;
    {
        let headers = req.headers_mut()?;
        headers.set("User-Agent", APP_USER_AGENT)?;
        headers.set("Content-Type", "application/json")?;
    }
    let mut res = Fetch::Request(req).send().await?;
    let status = res.status_code();
    let body = &res.text().await.unwrap_or_default();
    console_log!("[TG SendMessage] status:{} body:{}", status, body);
    Ok(res)
}
