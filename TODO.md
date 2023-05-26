
## Fixes
- split review_helper.xdc into
  - App info preview
  - Testing preview
- Migrate to SEA_QL
- Replication protocol
- Figure out testing
- Merge initial message and appstore
- Fix retrieving of receivin chat

## Make appstore happy path run:
1. [x] bot admin configures the bot with QRCode
2. [x] bot admin uses the bot's CLI to add some apps to the bot's App Store
3. [x] bot admin uses the bot's CLI to get the 1:1 contact verification QR of the bot
4. [x] bot admin share bot's verification-QR with users
6. [ ] click on download button, get the app in the chat, the download button changes to "downloading.." and
7. [ ] When the app was delivered, the state changes to "in chat"

## Make reviw happy path run:
1. [x] User sends webxdc into 1:1 chat with the bot.
2. [x] The bot creates a new group for review. 
3. [x] After all requirements have been met, the bot asks the bot wheter he wants to send it to review. 
4. [x] The Bot creates another group for review with some testers (3) an one publisher.
5. [x] After testing is complete, the publisher can pubslih the app to the appstore. 