## Appstore Bot for DeltaChat
Welcome to the DeltaChat Appstore Bot. You can start using it today by contacting it at this `<email>`. The appstore bot acts both as distribution platform as well as publishing service.

## Using the Appstore Bot
**Downloading Apps**: When sending a message to the bot, it will reply with the appstore webxdc. You can then clicke the `add` button and the bot will send you the requested webxdc.

**Publishing Apps**: The `appstore` also provides a platform for developers to publish their own applications. Here's a step-by-step guide on how to do it:

1. Send your webxdc to the bot.
2. The bot will then create a group chat for your submission and upon receiving all neccessary information a dedicated review chat.
5. Once all requirements are met, the publisher will publish your app to the appstore.

### App Publishing Requirements:
The following structure outlines the essential information needed for publishing an app:

```rust
Copy code
pub struct AppInfo {
    pub name: String,                    // Taken from manifest.toml
    pub author_name: String,             
    pub author_email: String,
    pub source_code_url: String,         // Taken from manifest.toml
    pub image: String,                   // Taken from manifest.toml
    pub description: String,             // Taken from manifest.toml
    pub version: String,                 // Taken from manifest.toml
}
```

## Setting Up the Bot

### Requirements

You need to have `rust` and `node` installed.
Optionally, have [pnpm](https://pnpm.io/) installed.

### Setup

Go to the `frontend` folder and install the packages using `pnpm install` or `pnpm i`.
If you do not have `pnpm` installed, you can run `npx pnpm install` to use `pnpm` 
without installing it globally or use `npm install` instead.
However, if you use `npm install`, the latest versions of dependencies will be installed
instead of the ones listed in the `frontend/pnpm-lock.yaml` file.

Build the frontend by running `npm run build` or `pnpm run build`.
This step creates a `bot-data` directory in the root of the repository
with files `appstore.xdc`, `review-helpr.xdc` and `submit-helper.xdc`.

To run the bot, set the environment variables
`addr` and `mail_pw` with the login and password
and use the command `start`:
```
addr=bot@example.org mail_pw=My_P4ssword cargo run -- start
```
The environment variables need to be set the first time you start the bot
and will be saved into the bot account database afterwards.

Optionally, import example applications into the bot:
```
mkdir import
cp example-xdcs/*.xdc import/
cargo run -- import
```

You can set the `RUST_LOG=info` environment variable to get detailed logging from the bot.

### Testing

Tests are using [pytest](https://pytest.org/).

To run the tests, first build the bot in debug mode `cargo build` which builds the binary.

To run the tests, you need to install [tox](https://tox.wiki/)
and set `DCC_NEW_TMP_EMAIL` environment variable with an account creation URL.

Executing `tox` will run the tests.

To develop the tests, create a development environment with `tox -e py --devenv env`
and activate it with `. env/bin/activate`. Afterwards run `pytest`.
