use anyhow::{anyhow, Result};

#[derive(Clone)]
pub struct Config {
    pub app_token: String,
    pub admin_token: String,
    pub admin_cookie: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let get = |k: &str| std::env::var(k).map_err(|_| anyhow!("missing env var {k}"));
        Ok(Self {
            app_token: get("SLACK_APP_TOKEN")?,
            admin_token: get("SLACK_ADMIN_TOKEN")?,
            admin_cookie: get("SLACK_ADMIN_COOKIE")?,
        })
    }
}
