
## Fixes
- [ ] split review_helper.xdc for review and submit chats
- [ ] Remove surrealdb
- [ ] Figure out testing
- [ ] App updating
- [ ] Improve error handling in review
- [ ] Fix conditionial building
- [ ] Close submit helper webxdc on submit
- [ ] Move bot files into dedicated folder

## Make appstore happy path run:
1. [x] bot admin configures the bot with QRCode
2. [x] bot admin uses the bot's CLI to add some apps to the bot's App Store
3. [x] bot admin uses the bot's CLI to get the 1:1 contact verification QR of the bot
4. [x] bot admin share bot's verification-QR with users
6. [ ] click on download button, get the app in the chat, the download button changes to "downloading.." and
7. [x] When the app was delivered, the state changes to "in chat"

## Make reviw happy path run:
1. [x] User sends webxdc into 1:1 chat with the bot.
2. [x] The bot creates a new group with the user where he forwards the webxdc to. It also send the helper wbxdc where things like name, description and so on can be seen and edited.
3. [] When all required fields are filled in, the user can send the webxdc to review.
4. [x] Upon review request (send from helper xdc), the bot creates a new group with some testers and one publisher.
5. The testers test the app and checkmark required tests in the helper webxdc.
6. [x] When all requirements are met, the publisher can publish the app to the appstore.