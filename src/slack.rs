use anyhow::{anyhow, Result};
use serde_json::Value;
use tracing::{info, warn};

use crate::config::Config;

#[derive(Clone)]
pub struct Slack {
    http: reqwest::Client,
    cfg: Config,
}

impl Slack {
    pub fn new(cfg: Config) -> Self {
        Self {
            http: reqwest::Client::new(),
            cfg,
        }
    }

    pub fn cfg(&self) -> &Config {
        &self.cfg
    }

    pub async fn open_socket(&self) -> Result<String> {
        let resp: Value = self
            .http
            .post("https://slack.com/api/apps.connections.open")
            .bearer_auth(&self.cfg.app_token)
            .send()
            .await?
            .json()
            .await?;
        if resp["ok"].as_bool() != Some(true) {
            return Err(anyhow!("apps.connections.open failed: {resp}"));
        }
        resp["url"]
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| anyhow!("no url in apps.connections.open response"))
    }

    /// Calls a Web API method as the admin user. Works with an OAuth user
    /// token (xoxp) or a browser session token (xoxc + `d` cookie).
    async fn admin_post(&self, method: &str, params: &[(&str, &str)]) -> Result<Value> {
        let mut req = self
            .http
            .post(format!("https://slack.com/api/{method}"))
            .bearer_auth(&self.cfg.admin_token)
            .form(params);
        if let Some(cookie) = &self.cfg.admin_cookie {
            req = req.header("Cookie", format!("d={cookie}"));
        }
        let resp: Value = req.send().await?.json().await?;
        if resp["ok"].as_bool() != Some(true) {
            let err = resp["error"].as_str().unwrap_or("unknown");
            return Err(anyhow!("{method} failed: {err}"));
        }
        Ok(resp)
    }

    pub async fn convert_to_public(&self, channel_id: &str) -> Result<()> {
        self.admin_post(
            "admin.conversations.convertToPublic",
            &[("channel_id", channel_id)],
        )
        .await?;
        Ok(())
    }

    pub async fn channel_managers(&self, channel_id: &str) -> Result<Vec<String>> {
        // Built-in "Channel Manager" role ID, as returned by
        // admin.roles.entity.listAssignments (not in Slack's public docs).
        const CHANNEL_MANAGER_ROLE: &str = "Rl0A";
        let resp = self
            .admin_post(
                "admin.roles.entity.listAssignments",
                &[("entity_id", channel_id), ("role_id", CHANNEL_MANAGER_ROLE)],
            )
            .await?;
        let users = resp["role_assignments"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|a| a["role_id"].as_str() == Some(CHANNEL_MANAGER_ROLE))
            .filter_map(|a| a["users"].as_array())
            .flatten()
            .filter_map(|u| u.as_str().map(str::to_owned))
            .collect();
        Ok(users)
    }

    pub async fn is_admin(&self, user_id: &str) -> Result<bool> {
        let resp = self.admin_post("users.info", &[("user", user_id)]).await?;
        let u = &resp["user"];
        Ok(u["is_admin"].as_bool().unwrap_or(false)
            || u["is_owner"].as_bool().unwrap_or(false)
            || u["is_primary_owner"].as_bool().unwrap_or(false))
    }

    /// Returns `(is_private, creator)`. With a user token, `conversations.info`
    /// only sees private channels the admin is in; for any other channel it
    /// fails, so we assume private with an unknown creator.
    pub async fn channel_info(&self, channel_id: &str) -> (bool, Option<String>) {
        match self
            .admin_post("conversations.info", &[("channel", channel_id)])
            .await
        {
            Ok(resp) => {
                let c = &resp["channel"];
                (
                    c["is_private"].as_bool() != Some(false),
                    c["creator"].as_str().map(str::to_owned),
                )
            }
            Err(e) => {
                warn!("{e:#}; assuming {channel_id} is private");
                (true, None)
            }
        }
    }

    pub async fn respond(&self, response_url: &str, text: &str) -> Result<()> {
        let resp = self
            .http
            .post(response_url)
            .json(&serde_json::json!({
                "response_type": "ephemeral",
                "text": text,
            }))
            .send()
            .await?;
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(anyhow!("response_url reply failed: {status} {body}"));
        }
        info!("ephemeral reply -> {status} {body}");
        Ok(())
    }
}
