This document should act as an entry point for new developers and the dc-team to get a 
bettor overview over the bots inner workings as well as what is implement and not at the moment.

## Structure
The bot currently uses four different chat types: `Shop`, `Submit`, `Review` and `Genesis`. Upon receiving a message or an `webxdcStatusUpdate` the bot will use the map-like `chat_to_chat_type` database table to get the correct chat-type for each `ChatId`. This happens in the `bot.rs` file where the bot creation/configuration and starting takes place. 
After the chat type is determined, the bot passes the message or `webxdcStatusUpdate` to the respective message handlers which live in `request_handlers/<chat_type>.rs`. These files contain all the logic needed to handle a message in one specfic chat. Most of the time the contained request handlers (be it for chat-messages or `webxdcStatusUpdates`) receive a `State` instance which holds the bot-state (for example a connection-pool to the db) and the Deltachat `Context` object which is neccessary to send messages, receive chats etc. etc.

## Chat types

- The `Shop` chat is the primarly 1:1 with a bot-user who wants to use webxdcs from the store.
  Upon receiving a 1:1 chat message _or_ when a `DC::Contact` is verified with a QR-code, the bot creates
  this kind of chat and sends the initial `store.xdc`. Currently implemented interaction is:
    - Updating the store.xdc
    - Receiving app updates
    - Searching the appstore
    - Downloading apps

- The `Submit` chat is created by the bot when he receives a webxdc in the `shop`-chat. This chat handles the submission of one singular webxdc app, indetified by it's `app_id`. When this chat is created, the original submitter is added to the chat, the submitted webxdc is forwarded and a helper xdc (submit-helper.xdc) is send to the chat.

- The `Review` chat is created by clicking the submit-button in the submit-helper.xdc (previous step). When creating such a chat, the bot chooses at most 3 testers and one publisher and adds them to the chat together with one forwarded instance of the submitted webxdc as well as the `review-helper.xdc`. When all tests are complete, the publisher can click on `publish` to finally release the webxdc to the store.

- The `Genesis` chat type acts as an administration group for the chat but at the moment it has no more functionality than letting users join the testers and publishers.

In general, to add a webxdc as a developer without CLI access the three consecutive chat types are used: 
1. `Shop`: To initiatie a submit.
2. `Submit`: To check all needed properties and start a review.
3. `Review`: To test a submitted webxdc and finally release it to the store.

To upgrade a webxdc it is intended to send a newer version to the `Submit` chat which will trigger a new
release cycle.

## App-Ids
The bot uses two kinds of ids: One is the databases `row-id` and the other one is the `app-id` taken from 
the webxdcs `manifest.toml`. Internally, only the `row-id` is used because `app-id` was just recently introduced and its not possible to create a PRIMARY-KEY column of type TEXT in an sqlite database. The `row-id` is used as an unambiguous indentifier for each `app-info` which is stored in the database.
The newly added `app-id` should only be used to uniqely identify newly added webxdcs and their different versions. When adding an webxdc, the `app-id` is used to distinguish between app-upgrading request and request which intially add a new webxdc to the store. Other than that no more use-cases are intended as of writing this.

## Updating webxdcs
It should be able to update webxdcs on two ways: 

1. By importing newer versions with the CLI
2. By updating them from the submit-chat.

For the MVP only the first option is needed but the general procedure should be the same.
For every added webxdcs the `app-id` and `version` are checked and a new  `AppInfo` is created and added to the database. If there already is an `AppInfo` with the same `app-id` but an older version, then this
older `AppInfo` will be invalidated (By setting active=false). 
When the frontend requests an update, the row-id of this deactivated webxdc should be
sent to the frontend. Implementation of this is not clear yet.

## Frontend
The frontend is build with `SolidJs` as this framework is compiled and produces very fast and more importantly small webxdcs. Most of the styling is done with uno-css which is a tailwind-like css utility framework. Only some exceptions are contained in the `index.sass` file. 
Currently three webxdcs are build by the frontdend: `store.xdc`, `submit-helper.xdc` and `review-helper.xdc`.
All of these work on some stripped version of the rusts `AppInfo` struct. Private fields like
`xdc-blob-dir` are removed, so that only the needed fields are send to the frontend.
On the frontend the `row-id` (AppInfo::id) is used to distinguish different AppInstances. The store.xdc for example uses this id to handle the caching.






