# slackbot.rs

A bot framework for [Slack](https://slack.com/), made in [Rust](http://www.rust-lang.org/).

## Usage

To make a bot using this framework, create a new cargo project and add the `slackbot.rs` dependency to `Cargo.toml`:

```toml
[dependencies.slackbot]
git = "https://github.com/augustt198/slackbot.rs"
```

Inside the main function, create a new `SlackBot` struct using `Slackbot::new(port: int)`:

```rust
let slackbot = SlackBot::new(8080);
slackbot.username   = Some(...);    // Your bot's username    (optional)
slackbot.icon_emoji = Some(...);    // Your bot's icon emoji  (optional)
slackbot.icon_url   = Some(...);    // Your bot's url emoji   (optional)
```

Any function with the `fn(&mut SlackCommand, &mut SlackResponse)` signature can be registered as a command:

```rust
fn test_command(cmd: &mut SlackCommand, resp: &mut SlackResponse) {
    resp.reply("Hello, world!"); 
}

slackbot.manager.register("test".to_string(), test_command);
```

Finally, start the bot:

```rust
slackbot.start();
```

---

Disclaimer: I have no idea what I'm doing, expect bad code.
