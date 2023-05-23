
- split review_helper.xdc into
  - App info preview
  - Testing preview
- Create tester _and_ creator chats
  - Tester chat interaction
  - Creator chat interaction
- Fix tester assignment of the bot itself
- Proper manifest & img for both xdcs
- Migrate to SEA_QL
- Set chat type for genesis group


## Make happy path run:
1. bot admin configures the bot with email + password
2. bot admin uses the bot's CLI to add some apps to the bot's App Store
3. bot admin uses the bot's CLI to get the 1:1 contact verification QR of the bot
4. bot admin share bot's verification-QR with users
5. users scan the bot's QR, the bot replies with the App Store users open the app store and see the list of apps, click on download button, get the app in the chat, the download button changes to "downloading.." and "app is in chat" when the app is sent