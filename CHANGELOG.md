# Change log

All notable changes to this project will be documented in this file.

## main branch

* Enable [“fat” link time optimization][lto] for release builds. On macOS, the
  resulting binary went from 8.9 MB (101 seconds) to 7.5 MB (123 seconds).

[lto]: https://doc.rust-lang.org/rustc/codegen-options/index.html#lto

## Release 0.0.1 (2025-09-03)

### Features

* Initial release.
