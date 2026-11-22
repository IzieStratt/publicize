use anyhow::{anyhow, Result};
use serde::Deserialize;
use serde_json::Value;
use tracing::info;

use crate::config::Config;

#[derive(Clone)]
pub struct Slack {
    http: reqwest::Client,
    cfg: Config,
}

#[derive(Deserialize)]
struct ApiOk {
    ok: bool,
    error: Option<String>,
}

impl Slack {
    pub fn new(cfg: Config) -> Self {
        Self {
            http: reqwest::Client::new(),
            cfg,
        }
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

    async fn admin_post(&self, method: &str, params: &[(&str, &str)]) -> Result<Value> {
        let mut form = params.to_vec();
        form.push(("token", &self.cfg.admin_token));
        let resp: Value = self
            .http
            .post(format!("https://slack.com/api/{method}"))
            .header("Cookie", format!("d={}", self.cfg.admin_cookie))
            .form(&form)
            .send()
            .await?
            .json()
            .await?;
        Ok(resp)
    }

    pub async fn convert_to_public(&self, channel_id: &str) -> Result<()> {
        let resp: ApiOk = self
            .http
            .post("https://slack.com/api/admin.conversations.convertToPublic")
            .bearer_auth(&self.cfg.admin_token)
            .header("Cookie", format!("d={}", self.cfg.admin_cookie))
            .form(&[("channel_id", channel_id)])
            .send()
            .await?
            .json()
            .await?;
        if resp.ok {
            Ok(())
        } else {
            Err(anyhow!(
                "convertToPublic failed: {}",
                resp.error.unwrap_or_else(|| "unknown".into())
            ))
        }
    }

    pub async fn channel_managers(&self, channel_id: &str) -> Result<Vec<String>> {
        const CHANNEL_MANAGER_ROLE: &str = "Rl0A";
        let resp = self
            .admin_post(
                "admin.roles.entity.listAssignments",
                &[("entity_id", channel_id), ("role_id", CHANNEL_MANAGER_ROLE)],
            )
            .await?;
        if resp["ok"].as_bool() != Some(true) {
            return Err(anyhow!("listAssignments failed: {resp}"));
        }
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

    pub async fn channel_creator(&self, channel_id: &str) -> Result<Option<String>> {
        let resp = self
            .admin_post("conversations.info", &[("channel", channel_id)])
            .await?;
        Ok(resp["channel"]["creator"].as_str().map(str::to_owned))
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
        info!("ephemeral reply -> {status} {body}");
        Ok(())
    }
}
