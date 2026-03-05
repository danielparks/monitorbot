# Change log

All notable changes to this project will be documented in this file.

## main branch

### Security

* Multiple security vulnerabilities in [aws-lc-rs]. Monitorbot does not use the crate directly, but it appears to be used by [rustls] for TLS. At worst, the vulnerability might allow someone to spoof a TLS-authenticated site.

  * [GHSA-hfpc-8r3f-gw53]: AWS-LC has PKCS7_verify Signature Validation Bypass
  * [GHSA-vw5v-4f2q-w9xf]: AWS-LC has PKCS7_verify Certificate Chain Validation Bypass
  * [GHSA-65p9-r9h6-22vj]: AWS-LC has Timing Side-Channel in AES-CCM Tag Verification

[aws-lc-rs]: https://github.com/aws/aws-lc-rs/
[rustls]: https://github.com/rustls/rustls
[GHSA-hfpc-8r3f-gw53]: https://github.com/advisories/GHSA-hfpc-8r3f-gw53
[GHSA-vw5v-4f2q-w9xf]: https://github.com/advisories/GHSA-vw5v-4f2q-w9xf
[GHSA-65p9-r9h6-22vj]: https://github.com/advisories/GHSA-65p9-r9h6-22vj

### Miscellaneous

* Enable [“fat” link time optimization][lto] for release builds. On macOS, the
  resulting binary went from 8.9 MB (101 seconds) to 7.5 MB (123 seconds).

[lto]: https://doc.rust-lang.org/rustc/codegen-options/index.html#lto

## Release 0.0.1 (2025-09-03)

### Features

* Initial release.
