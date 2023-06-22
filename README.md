## WebXDC store: discover and share the latest apps to your chats

**Status: ALPHA, use at your own risk**

Welcome to the WebXDC store (also xdc store for short) which currently consists of:

- **Store bot**: a Rust-implemented bot which operates on an e-mail
  address and sends out a store xdc app. 

- **Store xdc**: gets sent from the bot to users when they setup contact
  or send the bot a message.  The store xdc supports search and
  discovery of the **app index**, including forwarding/sharing an app into user-chosen chats. 

- The store bot command line interface allows to import apps to 
  make them available in the **app index** which is presented by the store xdc. 


## Getting started with sharing a first app to a chat group 

TODO: make bot available via sending a message to: xdcstore@testrun.org 

TODO: insert QR code here for setting up contact with the xdcstore bot

After establishing initial contact the bot will send a store xdc app which provides
the central point of interaction for users: 

1. Scroll the app index and hit the "share/sendToChat/forward"
   button on any app. 

2. Select the chat where you want to share the app.
   If you just want to test an app yourself select the "Saved Message" chat. 

3. The app will now appear in the selected chat in draft mode so that
   you can set a text message and send the app to your chat partners.

4. Everyone in the chat can now start the app. The other chat members don't
   need to interact with the store bot at all. 


## Usability notes regarding store bot xdc frontend 

- At the top right of the store xdc app you can trigger an update of the "app index" 
  to make sure you have the latest versions. 

- When hitting the "sendToChat/sendToChat/forward" button on any app for
  the first time a download will be triggered (using the send/receive message WebXDC APIs).

- For now, any message you send to the store bot will trigger it to send 
  the current store xdc in a new message. Later on we rather want to use 
  an update mechanism so there will only need to be a single store xdc app in
  the store bot chat. 

## Setting up the bot

### Installing pre-built binary releases

TODO: Download the latest release from https://github.com/webxdc/store/releases,
extract the compressed archive to the desired location, it containts the bot command line
program and required assets. In the following steps it is assumed that you added the program
to your PATH variable, but you can run the binary directly.

To check the program is installed correctly run:

```
xdcstore version
```

You should see the version code printed on the screen.

If you want to install from source code, read [Developing / Building from source](#developing--building-from-source)

### Running the bot

To run the bot, set the environment variables
`addr` and `mail_pw` with the login and password
and use the command `start`:

```
    addr=bot@example.org mail_pw=My_P4ssword xdcstore start
```

The environment variables need to be set the first time you start the bot
and will be saved into the bot account database afterwards.

You may set the `RUST_LOG=info` environment variable to get detailed logging from the bot.

### Importing apps

To import WebXDC applications into the app index:

```
    xdcstore import /path/to/xdc_apps/
```

where `/path/to/xdc_apps/` is a folder with WebXDC files you want to import.

### Per-app metadata

The store bot uses the following meta data for each xdc app,
noted down in rust-struct style for now: 

```rust
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

## Developing / building from source

### Requirements

You need to have installed:
- `rust` (https://www.rust-lang.org/tools/install)
- `node` (https://docs.npmjs.com/downloading-and-installing-node-js-and-npm)
- `pnpm` (https://pnpm.io/installation)

### Building the frontend

The bot uses WebXDC apps as "frontend" to interact with users.
Go to the `frontend` folder and install the packages using `pnpm install`.

Build the frontend by running `pnpm build`.
This step creates a `bot-data` directory in the root of the repository
with files `store.xdc`, `review-helper.xdc` and `submit-helper.xdc`.

### Building the bot

```
cargo build
```

the binary will be available in `./target/debug/` folder

While developing it is easier to build and run the bot in a single step with:

```
addr=bot@example.org mail_pw=My_P4ssword cargo run -- start
```

### Importing test apps

For testing, there are some apps in the folder `example-xdcs` you can import:

```
    mkdir imported_apps
    cp example-xdcs/*.xdc imported_apps/
    cargo run -- import imported_apps
```

### Building pull requests

The CI automatically builds releases tarballs for testing for every GitHub Pull Request branch, the
download link can be found in the CI checks details section at the bottom of the Pull Request page.

### Testing the frontend

While developing the frontend, run `pnpm dev` in the `frontend` folder to test your changes in a
small emulator with mock data.

### Running automated tests 

Tests are using [pytest](https://pytest.org/).

To run the tests, first build the bot in debug mode `cargo build` which builds the binary.

To run the tests, you need to install [tox](https://tox.wiki/)
and set `DCC_NEW_TMP_EMAIL` environment variable with an account creation URL.

Executing `tox` will run the tests.

To develop the tests, create a development environment with `tox -e py --devenv env`
and activate it with `. env/bin/activate`. Afterwards run `pytest`.

### Releasing

TODO: create the tarball and a GitHub release on version tags.

To create locally the distribution tarball run the script at `./scripts/dist.sh`
