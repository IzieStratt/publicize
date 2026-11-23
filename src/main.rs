mod config;
mod manager;
mod slack;

use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, info, warn};

use config::Config;
use slack::Slack;

const REFUSE_TEXT: &str =
    ":bleh: sorry it doesnt appear you're the cm or creator, if this seems wrong reach out to <@U09KKMHLS15>";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "publicize=info".into()),
        )
        .init();

    let cfg = Config::from_env()?;
    let slack = Slack::new(cfg);

    loop {
        if let Err(e) = run_once(&slack).await {
            error!("socket session ended: {e:#}");
        }
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        info!("reconnecting");
    }
}

async fn run_once(slack: &Slack) -> Result<()> {
    let ws_url = slack.open_socket().await?;
    let (mut ws, _) = tokio_tungstenite::connect_async(&ws_url).await?;
    info!("socket mode connected");

    while let Some(msg) = ws.next().await {
        let msg = msg?;
        let Message::Text(txt) = msg else { continue };
        let env: Value = serde_json::from_str(&txt)?;

        match env["type"].as_str() {
            Some("hello") => info!("hello from slack"),
            Some("disconnect") => {
                warn!("slack asked us to disconnect");
                break;
            }
            Some("slash_commands") => {
                if let Some(envelope_id) = env["envelope_id"].as_str() {
                    let ack = serde_json::json!({ "envelope_id": envelope_id });
                    ws.send(Message::Text(ack.to_string())).await?;
                }
                let slack = slack.clone();
                let payload = env["payload"].clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_slash(&slack, payload).await {
                        error!("slash handler error: {e:#}");
                    }
                });
            }
            other => info!("ignoring envelope type {other:?}"),
        }
    }
    Ok(())
}

async fn handle_slash(slack: &Slack, payload: Value) -> Result<()> {
    let channel_id = payload["channel_id"].as_str().unwrap_or_default();
    let user_id = payload["user_id"].as_str().unwrap_or_default();
    let response_url = payload["response_url"].as_str().unwrap_or_default();

    if channel_id.is_empty() || response_url.is_empty() {
        warn!("slash payload missing channel_id/response_url");
        return Ok(());
    }

    if slack.is_public(channel_id).await? {
        slack
            .respond(response_url, ":neocat: this channel already public")
            .await?;
        return Ok(());
    }

    if !manager::is_channel_manager(slack, user_id, channel_id).await? {
        slack.respond(response_url, REFUSE_TEXT).await?;
        return Ok(());
    }

    match slack.convert_to_public(channel_id).await {
        Ok(()) => {
            slack
                .respond(response_url, "This channel is now public.")
                .await?;
        }
        Err(e) => {
            warn!("convert failed for {channel_id}: {e:#}");
            slack
                .respond(response_url, "Couldn't convert this channel. Try again, or ping an admin.")
                .await?;
        }
    }
    Ok(())
}
