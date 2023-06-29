## Make store happy path run:
1. [x] bot admin configures the bot with QRCode
2. [x] bot admin uses the bot's CLI to add some apps to the bot's store 
3. [x] bot admin uses the bot's CLI to get the 1:1 contact verification QR of the bot
4. [x] bot admin share bot's verification-QR with users
6. [x] click on download button, get the app in the chat, the download button changes to "downloading.." and
7. [x] When the app was delivered, the state changes to "in chat"

## Make review happy path run:
1. [x] User sends webxdc into 1:1 chat with the bot.
2. [x] The bot creates a new group with the user where he forwards the webxdc to. It also send the helper wbxdc where things like name, description and so on can be seen and edited.
3. [x] When all required fields are filled in, the user can send the webxdc to review.
4. [x] Upon review request (send from helper xdc), the bot creates a new group with some testers and one publisher.
5. [x] The testers test the app and checkmark required tests in the helper webxdc.
6. [x] When all requirements are met, the publisher can publish the app to the store. 

## MVP
- [x] terminology refactoring: "app store" rebranding 
- [x] create usable and thorough README.md
- Consumption/discovery of apps: 
  - [x] search
  - [x] `sendToChat`
  - [x] manual app index update
- Onboarding workflow:
  - [x] create 1:1 chat with QR-invite
  - [x] accept 1:1 chats with inital message
- Store.xdc:
  - [x] notice version difference upon message receival from an older webxdc
  - [x] show an updated button
  - [x] send new webxdc in chat
- Submission: 
  - [x] importing apps from the CLI
- Deployment
  - [ ] stable deployment of xdcstore@testrun.org on bomba 
  - [ ] stable/tested way to re-deploy a newer version of storebot 

## Further work and improvements
- [ ] Close submit helper webxdc on submit (core support needed)
- [ ] Handle upgrading for webxdcs in submit-chats
- [ ] Fordward messages between chats
- [ ] In-place store updating (core support needed)

## Testing

### Python Based
- [x] Test welcoming message
- [x] Test updating
  - [x] Requesting empty updates
- [x] Test importing 
  - [x] Test importing example xdcs
  - [x] Initial setup message contains webxdcs
- [x] Test `version` subcommand
  - [x] produces same output for message-based usage as well as CLI
- [x] Test downloading
  - [x] Test downloading an existing app
  - [x] Test downloadin a non-existing app
- [x] Test outdated bot-xdcs

### Rust Based
- [x] Test for close to every DB-function
- [x] Importing webxdcs
  - [x] Adding to appindex
  - [x] Upgrading appindex
  - [x] Keeping unchanged files untouched

### Frontend
Integration tests do need a _lot_ of setup work.
All API interaction is pretty comprehensively documented in python tests.

