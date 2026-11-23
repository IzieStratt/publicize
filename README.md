# publicize

## USAGE
run `/publicize` in a private channel, no need to invite anyone. you just gotta be either a:
- workspace/org owner or admin (finally a way for mobile!)
- a channel manager in the channel
- channel creator (if there are no managers)

## who made dis?
https://izie.top did

## setup
1. create the app from `manifest.json` and install it **org-wide** (the `admin.*` methods only work on an Enterprise Grid org install)
2. the installing user must be an org admin/owner. their user token (`xoxp-…`) is what flips the channel. bot tokens get `not_allowed_token_type`
3. set env vars:
   - `SLACK_APP_TOKEN`: app-level token (`xapp-…`, `connections:write`) for socket mode
   - `SLACK_ADMIN_TOKEN`: the admin's user token (`xoxp-…`). a browser session token (`xoxc-…`) works too
   - `SLACK_ADMIN_COOKIE`: only needed with an `xoxc` token (the `d` cookie value)
   - `PUBLICIZE_CONTACT` (optional): user ID to mention when someone gets refused
4. `cargo run --release`

dry-run the permission check without converting anything: `cargo run -- check <channel_id> <user_id>`
