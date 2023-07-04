
## The core `bot.rs` and the chats it manages 

The Bot maintains 1:1 chats with each of its users 
who can get in contact via a QR code or by sending an arbitrary message. 

The bot currently uses two different chat types: `Store` and `Genesis`: 

- The `Genesis` chat type acts as an administrative group for the chat 
  but at the moment it has no particular functionality. 
  Later, we want to arrange app testing and other administrative tasks in it. 

- The `Store` chat is the primary 1:1 chat with a bot-user 
  who wants to discover, download and use webxdcs from the store.
  Upon receiving a 1:1 chat message _or_ when a `DC::Contact` is verified with a QR-code, 
  the bot creates a `Store` type chat and sends the initial `store.xdc` frontent to the user. 
  Currently implemented Bot/Frontend interactions are: 

    - Updating the store.xdc
    - Receive updates to the app index
    - Searching the app index 
    - Downloading apps
    - Sharing an app to a chat ("installing") 

Upon receiving a message or a `webxdcStatusUpdate` the bot will use 
the map-like `chat_to_chat_type` database table 
to get the correct chat-type for each `ChatId`. 
After the chat type is determined, the bot passes the message or `webxdcStatusUpdate` 
to the respective message handlers which live in `request_handlers/<chat_type>.rs`. 
Request handlers (be it for chat messages or `webxdcStatusUpdates`)
receive a `State` instance which holds the bot-state (for example a connection pool to the DB)
and the Deltachat `Context` object which is necessary to send messages receive chats etc. etc.

## App Ids

The bot uses two kinds of ids: One is the database's `row-id` and the other one is the `app-id` 
taken from the webxdcs `manifest.toml`. Internally, currently the `row-id` is used 
as an unambiguous identifier for each `app-info` which is stored in the database. 
The newly added `app-id` should only be used to uniquely identify newly added webxdcs and their 
different versions. When adding a webxdc, the `app-id` is used to distinguish between 
app-upgrading requests and requests which initially add a new webxdc to the store. 
Other than that no more use cases are intended as of writing this.

## Updating the App Index

Updating the App Index happens by importing newer versions of webxdc app files with the CLI. 
For every added webxdc app the `app-id` and `version` are checked and a new `AppInfo` is created 
and added to the database. 
If there already is an `AppInfo` with the same `app-id` but an older version, 
then this older `AppInfo` will be invalidated. 

Every change to the app index increases a "serial" 
which the frontend and bot use for synchronization, see `Synchronizing the App Index`. 

## Frontend

The frontend is built with `SolidJs` as this framework is compiled and produces very fast 
and more importantly small webxdc apps. Most of the styling is done with uno-css 
which is a tailwind-like CSS utility framework. 
Only some exceptions are contained in the `index.sass` file. 
Currently, one webxdc app is built by the frontend: `store.xdc`. 
It works on some stripped version of the Rust `AppInfo` struct. 
Private fields like `xdc-blob-dir` are removed, so that only the 
needed fields are sent to the frontend.
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
