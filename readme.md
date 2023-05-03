## DC-Github-Bot

A Deltachat-bot which works as bridge between Deltachat and Github Webhoooks

### Usage

Users can interact with the bot by issuing `commands`.
All `commands` have to be prefixed with `gh` and can be of the following form:

```rust
enum Cli {
    /// Subscribe to an event
    Subscribe {
        /// Id of the repository
        repo: usize,

        Pr {
            #[arg(value_enum)]
            pr_action: PRAction,
        },
        Issue {
            #[arg(value_enum)]
            issue_action: IssueAction,
        },
    },

    /// Unsubscribe from an event
    Unsubscribe {
        /// Id of the repository
        repo: usize,

        Pr {
            #[arg(value_enum)]
            pr_action: PRAction,
        },

        Issue {
            #[arg(value_enum)]
            issue_action: IssueAction,
        },
    },

    // Change and list supported repositories
    Repositories {
        // List all available repositories
        List,

        // Add a webhook for a new repository
        Add {
            // Name of repo owner (user or organisation)
            owner: String,

            // Name of repository
            repository: String,

            // REST-Api key
            api_key: String,
        },

        // Remove a repositories webhook
        Remove {
            // Id of repository to remove
            repository: usize,

            // REST-Api key
            api_key: String,
        },
    },
}
```

### Examples

**Adding a repository**:

```
gh repositories add septias github-bot ghp_xyp
```

where `ghp_xyp` is a github rest-api-key that can be created like [this](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/creating-a-personal-access-token).

**Adding an event listener**:

```
gh subscribe 123534 issue opened
```

where 123534 is a valid repo taken from:

**Listing all repositories**:

```
gh repositories list
```

### Architecture

- The bot has to be hosted under a public IP to be able to receive github webhooks.
- The file `server.rs` spins up a `tide` webserver listening on port `0.0.0.0:8080/receive`
- The repository webhook sends all events to this endpoint where they are parsed and processed.
- After receiving a webhook event, the bot distributes it to all listeners.
- The client requests are parsed using `clap`.

### Files

```
.
├── src
│ ├── bot.rs       // bot code
│ ├── db.rs        // surrealdb-api
│ ├── main.rs      // spin up bot
│ ├── parser.rs    // CLI definition using `clap`
│ ├── queries      // some of the sql-queries used in `db.rs`
│ ├── rest_api.rs  // interaction with the github rest-api
│ ├── server.rs    // spin up `tide` server
│ ├── shared.rs    // some types
│ └── utils.rs
```

### Development
Start the bot like this: 
```
RUST_LOG=info addr=<add> mail_pw=<pw> cargo r
```
where `<addr>` and `<pw>` are some valid login credentials for an email-server.

#### Testing
It comes in handy to send webhook-events manually with curl:
```bash
curl -X POST --data "mock/issue_open.json" localhost:8080/receive --header "X-GitHub-Event: issues"
```


### Further improvement

- Don't allow users to register listeners twice
  - this gets rejected internally, but is not shown to user
