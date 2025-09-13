# Monitor a web page for changes

This is very pre-release.

```diff
❯ cargo run -- http://demon.horse/hireme/
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.16s
     Running `target/debug/monitorbot 'http://demon.horse/hireme/'`

 I have extensive experience taking projects from conception to production. I spent years helping small businesses define and build web applications and tools. I’ve both built applications from the ground up and added features to old, hairy codebases. I know when to go fast and dirty, and when to go slow and clean.
+
+I’m looking for a non-management technical leadership position. I’m flexible about what languages are involved, but I’m particularly interested in writing Rust, Python, Go, or JavaScript. I hope to work closely with other engineers —I believe that teams succeed when members support each other and grow together.

 ### 2015 to 2019 —Puppet —Senior Software Engineer, Manager of Engineering
```

## Installation

```sh
cargo install monitorbot
```

If you have [`cargo binstall`][binstall], you can use it to download and install
a binary:

```sh
cargo binstall monitorbot
```

Finally, you can download binaries directly from the [GitHub releases
page][releases]. Just extract the archive and copy the file inside into your
`$PATH`, e.g. `/usr/local/bin`. The most common ones are:

  * Linux: [x86-64](https://github.com/danielparks/monitorbot/releases/latest/download/monitorbot-x86_64-unknown-linux-gnu.tar.gz),
    [ARM](https://github.com/danielparks/monitorbot/releases/latest/download/monitorbot-aarch64-unknown-linux-musl.tar.gz)
  * macOS: [Intel](https://github.com/danielparks/monitorbot/releases/latest/download/monitorbot-x86_64-apple-darwin.tar.gz),
    [Apple silicon](https://github.com/danielparks/monitorbot/releases/latest/download/monitorbot-aarch64-apple-darwin.tar.gz)
  * [Windows on x86-64](https://github.com/danielparks/monitorbot/releases/latest/download/monitorbot-x86_64-pc-windows-msvc.zip)


## Rust Crate

[![docs.rs](https://img.shields.io/docsrs/monitorbot)][docs.rs]
[![Crates.io](https://img.shields.io/crates/v/monitorbot)][crates.io]
![Rust version 1.85+](https://img.shields.io/badge/Rust%20version-1.85%2B-success)

## Development status

This is in active development. I am open to [suggestions][issues].

## License

Unless otherwise noted, this project is dual-licensed under the Apache 2 and MIT
licenses. You may choose to use either.

  * [Apache License, Version 2.0](LICENSE-APACHE)
  * [MIT license](LICENSE-MIT)

### Contributions

Unless you explicitly state otherwise, any contribution you submit as defined
in the Apache 2.0 license shall be dual licensed as above, without any
additional terms or conditions.

[docs.rs]: https://docs.rs/monitorbot/latest/monitorbot/
[crates.io]: https://crates.io/crates/monitorbot
[binstall]: https://github.com/cargo-bins/cargo-binstall
[releases]: https://github.com/danielparks/monitorbot/releases
[issues]: https://github.com/danielparks/monitorbot/issues
