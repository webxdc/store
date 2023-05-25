
- split review_helper.xdc into
  - App info preview
  - Testing preview
- Create tester _and_ creator chats
  - Tester chat interaction
  - Creator chat interaction
- Migrate to SEA_QL

## Make happy path run:
1. [x] bot admin configures the bot with QRCode
2. [x] bot admin uses the bot's CLI to add some apps to the bot's App Store
3. [x] bot admin uses the bot's CLI to get the 1:1 contact verification QR of the bot
4. [x] bot admin share bot's verification-QR with users
5. [x] users scan the bot's QR, the bot replies with the App Store users open the app store and see the list of apps, click on download button, get the app in the chat, the download button changes to "downloading.." and "app is in chat" when the app is sent