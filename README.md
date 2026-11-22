# publicize

A small Slack bot that lets a channel manager turn their own private channel
public with a slash command, instead of filing a ticket with an admin.

On an Enterprise Grid org, converting a private channel to public is an admin
action (`admin.conversations.convertToPublic`). This runs that call on behalf of
people who are allowed to make the decision for their channel but aren't org
admins: the channel's managers and its creator.

## Usage

Run `/publicize` in the private channel you want to open up. The bot checks that
you're allowed to, converts the channel, and replies to you only. If you're not
a manager or the creator it refuses and tells you who to ask.

## Who's allowed

In order:

1. Workspace/org admins and owners.
2. Anyone holding the channel-manager role on that channel.
3. The channel's creator (used when a channel has no managers assigned).

Managers are read from the `Rl0A` role via `admin.roles.entity.listAssignments`.

## Running it

It connects over Socket Mode, so there's no public URL to host.

```
cp .env.example .env   # fill in the three values
cargo run
```

| Variable | What it is |
| --- | --- |
| `SLACK_APP_TOKEN` | app-level token (`xapp-…`) with `connections:write` |
| `SLACK_ADMIN_TOKEN` | admin user token (`xoxc-…`) used for the admin API calls |
| `SLACK_ADMIN_COOKIE` | the `d` cookie value (`xoxd-…`) paired with that token |

The Slack app itself only needs the `commands` bot scope; the reply goes back
through the slash command's `response_url`. The privileged calls run on the
admin user token, which has to belong to an org/workspace admin — a bot token
can't make them.

The app manifest is in `manifest.json`.
