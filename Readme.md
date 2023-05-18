## Appstore Bot

The official appstore bot for deltachat. You can contact it at `<email>`.


## Usage
- Bot sends `appstore` to you where you can download apps

### Publishing
- In the `appstore` you can publish your new apps. 
  1. Open the publish page in the appstore.
  2. Give simple information like name and description.
  3. Bot will create a group for review.
    - The group contains mutliple testers which will test the app on different devices.
    - The group contains one reviewer, who will publish the app if it meets the requirements.
  4. Post your bundled webxdc.
  5. Give neccessary information and publish it to meet the testers requirements.
  6. The publisher will publish your app to the appstore.

#### Requirements:
The app needs the following information:

```rust
pub struct AppInfo {
    pub name: String,                    // taken from manifest.toml
    pub author_name: String,             // created by bot from contact
    pub author_email: Option<String>,    // created by bot from contact
    pub source_code_url: Option<String>, // taken from manifest.toml
    pub image: Option<String>,           // taken from manifest.toml
    pub description: String,             // given in submit formular
    pub xdc_blob_dir: Option<PathBuf>,   // created by bot from contact
    pub version: Option<String>,         // taken from manifest.toml
}
```

## Setup
Clone this repository and start the bot like this:
```
addr="<email>" mail_pw="<password>" RUST_LOG=info cargo r
```

The bot will then ask for a administrator email address and after it was given it, it will create two groups: `Publisher Group` and `Tester Group`. 

`Publisher Group`: A group of trusted entities which will finally add an app to the appstore.
`Tester Group`: A group of testers, maybe from the community which can test the apps on their devices.

You can add new members to the different roles by adding them to the group chats.