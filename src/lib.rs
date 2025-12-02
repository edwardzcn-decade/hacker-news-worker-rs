pub mod api;
pub mod config;
pub mod kvm;
pub mod router;
pub mod scheduled;

use axum::{routing::get, Router};

use tower_service::Service;

use worker::*;

fn router() -> Router {
    Router::new()
        .route("/", get(router::get_root))
        .route("/about", get(router::get_about))
        .route("/blog", get(router::get_blog))
        .route("/forward/{key}", get(router::get_forward_key))
        .route("/forward/{item}/{id}", get(router::get_forward_item))
        .fallback(router::fallback_handler)
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

#[event(scheduled)]
async fn scheduled(event: ScheduledEvent, _env: Env, _ctx: ScheduleContext) {
    console_log!(
        "[Scheduled] Scheduled event triggered at: {}",
        js_sys::Date::new_0().to_iso_string()
    );
    match event.cron().as_str() {
        "*/10 * * * *" => {
            if let Err(e) = scheduled::run_telegram_job(_env, None).await {
                // TODO consider catch error here or inside
                console_error!("{:?}", e);
            }
        }
        "30 9 * * mon,wed,fri" => {
            scheduled::test_job_handler().await;
        }
        _ => {
            console_warn!("[Scheduled] ⚠️ Mismatch cron expression: {}. https://github.com/edwardzcn-decade/hacker-news-worker/tree/main?tab=readme-ov-file#scheduled-jobs", event.cron());
        }
    }
}
