## Webxdc app store 

**Status: ALPHA, use at your own risk**

Welcome to the Webxdc app store (xdcstore for short) which currently consists of: 

- a Rust-implemented bot which operates on an e-mail address for Delta Chat users 

- a "store.xdc" sent from the bot to users supporting search and
  discovery of apps, including forwarding/sharing them with user-chosen chats 

- a command line interface to import/make apps available 


## Getting started with sharing a first app to a chat group 

TODO: make bot available via sending a message to: xdcstore@testrun.org 

TODO: insert QR code here for setting up contact with the xdcstore bot

After establishing initial contact the bot will send a "xdc store" app which provides
the central point of interaction for users: 

- Scroll the app index list and hit the "share/sendToChat/forward"
  button on any app. 

- Select the chat where you want to share the app.

- The webxdc app will appear in draft mode in the selected chat so that
  you can modify the message and send it to your chat partners.

- Everyone in the chat can now start the app but the other chat members don't
  need to interact with the store bot at all. 

- If you just want to test an app for yourself select the "Saved Message"
  chat for sharing the app. 


## Usability notes regarding store bot xdc frontend 

- At the top right of the xdc store app you can trigger an update of the "app index" 
  to make sure you have the latest versions. 

- When hitting the "sendToChat/sendToChat/forward" button on any app for
  the first time a download will be triggered (using the send/receive message webxdc APIs). 

- For now, any message you send to the store bot will trigger it to send 
  the current store xdc in a new message. Later on we rather want to use 
  an update mechanism so multiple xdc store messages will be avoided. 

## Setting Up the Bot

### Requirements

You need to have `rust` and `node` installed.
Optionally, have [pnpm](https://pnpm.io/) installed.

### Setting up the bot 

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

You may set the `RUST_LOG=info` environment variable to get detailed logging from the bot.

### Importing apps

To import example applications into the bot:

```
    mkdir import
    cp example-xdcs/*.xdc import/
    cargo run -- import
```

### Per-App metadata 

The store bot uses the following meta data for each xdc app,
noted down in rust-struct style for now: 

```
    // Fields specified in https://docs.webxdc.org/spec.html#the-manifesttoml-file
    pub name: String,                    // Taken from manifest.toml
    pub image: String,                   // Taken from .xdc file
    pub source_code_url: Option<String>, // Taken from manifest.toml

    // Fields not specified yet but required for store bot purposes 
    pub description: String,             // Taken from manifest.toml
    pub app_id: String,                  // Taken from manifest.toml 
    pub version: String,                 // Taken from manifest.toml 

    // Derived data (not coming from manifest.toml or .xdc file) 
    pub submitter_uri: Option<String>,   // determined by bot during interaction 
    pub submission_date: Date,           // time of submission, filled out by bot
```

Notes: 

- The `app_id` and `version` fields are used by the bot to sort submitted apps so that 
  the store xdc can offer the latest version of each app to users. 

- The `app_id` field MUST be ASCII-alphanumberic with only `.` allowed as special character. 
  Casing doesn't matter and will be ignored by the bot when doing `app_id` comparisons. 

- The `version` field MUST adhere to https://semver.org/ -- i.e. be a 

- The `submitter_uri` can be a URL, a mailto or xmpp URI and is
  determined by the bot at submission time which is also recorded in `submission_date`. 

- We do not define "authorship" yet because it likely is better to
  introduce together with code signing so that the information is authenticated. 
  However, the `source_code_url` already provides an (unauthenticated) 
  way to point to the author(s). 


### Running automated tests 

Tests are using [pytest](https://pytest.org/).

To run the tests, first build the bot in debug mode `cargo build` which builds the binary.

To run the tests, you need to install [tox](https://tox.wiki/)
and set `DCC_NEW_TMP_EMAIL` environment variable with an account creation URL.

Executing `tox` will run the tests.

To develop the tests, create a development environment with `tox -e py --devenv env`
and activate it with `. env/bin/activate`. Afterwards run `pytest`.
