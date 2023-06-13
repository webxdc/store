
## Fixes
- [ ] Add testing
- [ ] App updating
- [ ] Close submit helper webxdc on submit
- [ ] What happens if people think they should upgrade-submit their apps into the 1:1 chat?
- [ ] Handle upgrade-submit in submit-chats.

## Make appstore happy path run:
1. [x] bot admin configures the bot with QRCode
2. [x] bot admin uses the bot's CLI to add some apps to the bot's App Store
3. [x] bot admin uses the bot's CLI to get the 1:1 contact verification QR of the bot
4. [x] bot admin share bot's verification-QR with users
6. [x] click on download button, get the app in the chat, the download button changes to "downloading.." and
7. [x] When the app was delivered, the state changes to "in chat"

## Make reviw happy path run:
1. [x] User sends webxdc into 1:1 chat with the bot.
2. [x] The bot creates a new group with the user where he forwards the webxdc to. It also send the helper wbxdc where things like name, description and so on can be seen and edited.
3. [x] When all required fields are filled in, the user can send the webxdc to review.
4. [x] Upon review request (send from helper xdc), the bot creates a new group with some testers and one publisher.
5. [x] The testers test the app and checkmark required tests in the helper webxdc.
6. [x] When all requirements are met, the publisher can publish the app to the appstore.