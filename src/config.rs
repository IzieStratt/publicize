use anyhow::{anyhow, Result};

#[derive(Clone)]
pub struct Config {
    pub app_token: String,
    pub admin_token: String,
    /// Only needed when SLACK_ADMIN_TOKEN is a browser session token (xoxc).
    pub admin_cookie: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let get = |k: &str| std::env::var(k).map_err(|_| anyhow!("missing env var {k}"));
        Ok(Self {
            app_token: get("SLACK_APP_TOKEN")?,
            admin_token: get("SLACK_ADMIN_TOKEN")?,
            admin_cookie: std::env::var("SLACK_ADMIN_COOKIE").ok().filter(|c| !c.is_empty()),
        })
    }
}
