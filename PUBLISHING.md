
## Publishing app to the store

Status: pre-Alpha 

The main current way to inject apps into the store is via the command line tool: 

```
    provide CLI example 
```

Moreover, the xdc store preliminarily provides a secondary way to submit applications: 

1. Send your webxdc to the bot.
2. The bot will create a group chat for your submission and upon receiving all neccessary information a dedicated review chat.
3. Once all requirements are met, and someone in the review chat confirms the submission
   the app will become available in the app index the store xdc makes available to users. 

### App Publishing Requirements

Here we record the metadata for each application submission that we intend 
to keep about each submitted app version, noted down in Rust-struct style: 

```rust
Copy code
pub struct AppInfo {
    // Fields specified in https://docs.webxdc.org/spec.html#the-manifesttoml-file
    pub name: String,                    // Taken from manifest.toml
    pub image: String,                   // Taken from .xdc file
    pub source_code_url: Option<String>, // Taken from manifest.toml

    // Fields not yet specified yet but needed/useful for store bot 
    pub description: String,             // Taken from manifest.toml
    pub app_id: String,                  // Taken from manifest.toml 
    pub version: String,                 // Taken from manifest.toml 

    pub author_uri: Option<String>,      // Taken from manifest.toml

    // derived data (not coming from manifest or xdc) 
    pub submitter_uri: Option<String>,   // determined by bot during interaction 
    pub submission_date: Date,           // filled out by bot 
}
```

Notes: 

- `app_id` and `version` fields are used by the bot to sort submitted apps so that 
  each app occurs with the latest version. 

- `app_id` MUST be alphanumberic && case-insensitive and only "." is
  allowed as special character 

- `version` MUST adhere to https://semver.org/ (for now) 

