This document should act as an entry point for new developers and the dc-team to get a
bettor overview of the bots' inner workings as well as what is implemented and not at the moment.

## Structure
The bot currently uses four different chat types: `Store`, `Submit`, `Review`, and `Genesis`. 
Upon receiving a message or a `webxdcStatusUpdate` the bot will use the map-like `chat_to_chat_type`
database table to get the correct chat-type for each `ChatId`. 
This happens in the `bot.rs` file where the bots' creation/configuration and starting take place. 
After the chat type is determined, the bot passes the message or `webxdcStatusUpdate` to the respective message handlers which live in `request_handlers/<chat_type>.rs`. 
These files contain all the logic needed to handle a message in one specific chat. 
Most of the time the contained request handlers (be it for chat messages or `webxdcStatusUpdates`)
receive a `State` instance which holds the bot-state (for example a connection pool to the DB)
and the Deltachat `Context` object which is necessary to send messages receive chats etc. etc.

## Chat Types
These are the four different chat types of which only `Store` is part of the MVP:

- The `Store` chat is the primary 1:1 with a bot-user who wants to use webxdcs from the store.
  Upon receiving a 1:1 chat message _or_ when a `DC::Contact` is verified with a QR-code, 
  the bot creates this kind of chat and sends the initial `store. xdc`. 
  Currently implemented interaction is:
    - Updating the store.xdc
    - Receive updates to the app index
    - Searching the store
    - Downloading apps

- The `Submit` chat is created by the bot when he receives a webxdc in the `store`-chat. 
  This chat handles the submission of one singular webxdc app, identified by its `app_id`. 
  When this chat is created, the original submitter is added to the chat, 
  the submitted webxdc is forwarded and a helper xdc (submit-helper.xdc) is sent to the chat.

- The `Review` chat is created by clicking the submit button in the `submit-helper.xdc` (previous step). 
  When creating such a chat, the bot chooses at most 3 testers and one publisher and adds them to the chat 
  together with one forwarded instance of the submitted webxdc as well as the `review-helper.xdc`. 
  When all tests are complete, the publisher can click on `publish` to finally release the webxdc to the store.

- The `Genesis` chat type acts as an administrative group for the chat 
  but at the moment it has no more functionality than letting users join the testers and publishers.

In general, to add a webxdc as a developer without CLI access the three consecutive chat types are used: 
1. `Store`: To initiate a submission.
2. `Submit`: To check all needed properties and start a review.
3. `Review`: To test a submitted webxdc and finally release it to the store.

To upgrade a webxdc it is intended to send a newer version to the `Submit` chat 
which will trigger a new release cycle.

## App Ids
The bot uses two kinds of ids: One is the database's `row-id` and the other one is the `app-id` 
taken from the webxdcs `manifest.toml`. Internally, currently the `row-id` is used 
as an unambiguous identifier for each `app-info` which is stored in the database. 
The newly added `app-id` should only be used to uniquely identify newly added webxdcs and their 
different versions. When adding a webxdc, the `app-id` is used to distinguish between 
app-upgrading requests and requests which initially add a new webxdc to the store. 
Other than that no more use cases are intended as of writing this.

## Updating the App Index
Updating the App Index should work in two ways:

1. By importing newer versions with the CLI.
2. By updating them from a `Submit` chat.

For the MVP only the first option is needed but the general procedure should be the same.
For every added webxdcs the `app-id` and `version` are checked and a new `AppInfo` is created 
and added to the database. If there already is an `AppInfo` with the same `app-id` but an older version, 
then this older `AppInfo` will be invalidated (By setting active=false).
Also, a serial number is increased with every change to enable partial updates for the frontend. 
For more information look at `Synchronizing the App Index`

## Frontend
The frontend is built with `SolidJs` as this framework is compiled and produces very fast 
and more importantly small webxdcs. Most of the styling is done with uno-css 
which is a tailwind-like CSS utility framework. 
Only some exceptions are contained in the `index.sass` file. 
Currently, three webxdcs are built by the frontend: 
`store.xdc`, `submit-helper.xdc`, and `review-helper.xdc`.
All of these work on some stripped version of the Rust `AppInfo` struct. 
Private fields like `xdc-blob-dir` are removed, so that only the needed fields are sent to the frontend.
On the frontend, the `row-id` (AppInfo::id) is used to distinguish different AppInstances. 
The store.xdc for example uses this id to handle the caching.

## Synchronizing the App Index
The app index describes the list of apps that are shown in the frontend webxdc.
When the store.xdc frontend is initially sent to a user's device, 
the bot also sends a `webxdcStatusUpdate` containing the current list of active `AppInfos` 
and the latest serial.
When the store.xdc frontend requests new updates it sends its current app-index serial 
and the bot will send any new `AppInfos` with a serial greater than that last seen serial. 
The store.xdc will then add them to the frontend app index along with the newest serial number. 
Only webxdcs that have been removed from the index will be sent in a special - yet to implement - field.

--- 

This project is still under heavy development by the webxdc working group. To see the current work go to https://github.com/orgs/deltachat/projects/61/views/1.