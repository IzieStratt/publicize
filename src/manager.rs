use anyhow::Result;

use crate::slack::Slack;

pub async fn is_channel_manager(
    slack: &Slack,
    user_id: &str,
    channel_id: &str,
    creator: Option<&str>,
) -> Result<bool> {
    if slack.is_admin(user_id).await? {
        return Ok(true);
    }

    let managers = slack.channel_managers(channel_id).await?;
    if !managers.is_empty() {
        return Ok(managers.iter().any(|m| m == user_id));
    }

    Ok(creator == Some(user_id))
}
