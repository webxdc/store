## Webxdc app store 

**Status: ALPHA, use at your own risk**

Welcome to the Webxdc app store (xdcstore for short) which currently consists of: 

- a Rust-implemented bot which operates on an e-mail address for Delta Chat users 

- a "store.xdc" for users to search and discover apps and to
  forward/share them with user-selected chats 

- a command line interface to import/make apps available 


## Getting started with sharing a first app to a chat group 

TODO: make bot available via sending a message to: xdcstore@testrun.org 

TODO: insert QR code here for setting up contact with the xdcstore bot

After establishing initial contact the bot will send a "xdc store" app which provides
the central point of interaction for users: 

- scroll the app index list and hit the "share/sendToChat/forward" button on any app 
  of your liking

- select the chat where you want to share the app 

- the webxdc app will appear in draft mode in the selected chat so that
  you can modify the message and send it to your chat partners 

- everyone in the chat can now start the app (the other chat members don't
  need to talk to the store bot or use the xdc). 

- if you just want to test it for yourself select the "Saved Message"
  chat for sharing the app 

## Some notes regarding the store bot 

- at the top right of the xdc store app you can trigger an update of the "app index" 
  to make sure you have the latest versions 

- when hitting the "sendToChat/sendToChat/forward" button on any app for
  the first time a download will be triggered (using the send/receive message webxdc APIs). 

- for now, any message you send to the store bot will trigger it to send 
  the current store xdc in a new message. Later on we rather want to use 
  an update mechanism so multiple xdc store messages will be avoided. 

- see `PUBLISHING.md` for some preliminiary info about alternative ways
  to submit application instead of the current admin-CLI importing one 

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
