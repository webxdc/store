

# high level topics about bot store dev+publishing 

- terminology refactoring: "app store" rebranding, holger to suggest a rewrite of
  the README with fitting terminology 

- delta chat spearheading efforts to make the store used by many people 
  to determine what needs to happen with messenger WebXDC APIs or implementation
  but cheogram should at least in principle be able to use bot/xdc store 
  even if it requires some extra future work. 

- MVP for xdc store:
  - **implement terminology refinements**
  - consumption/disocvery of apps: search + "sendToChat" + manual app index update 
  - onboarding work flow: either from QR code or via the e-mail adddress
    (just send any message in a 1:1 chat) 
  - store.xdc: should always show a version mismatch (bot delivers a
    differnet store.xdc version than the one currently in use/requesting things) 
    if there is a version mismatch, there is an update button in the store.xdc
    that will (for now) send a new message with the new store.xdc to the bot chat 
  - **submission: only importing apps from the CLI**
  - stable deployment of xdcstore@testrun.org on bomba 
    stable/tested way to re-deploy a newer version of storebot 
    for example: first deploy to xdcstore-staging@testrun.org 

- core prio: storexdc update mechanism (a core-level replace-attachment-xdc update mechanism) 

- submission:
  app-id? 
  author/author-contact-URI: to become part of manifest 
  what is currently "author" should be called "submitter" 
