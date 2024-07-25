# Hyperborealib

Library that implements the protocol and some important components needed for applications development.

This library provides relatively low-level types. It's better to use [hyperelm](https://github.com/HyperboreaHQ/hyperelm) for software development instead.

## Available features

1. General interfaces for data transforming (encryption, compression, encoding).
    - AES-256-GCM
    - ChaCha20-Poly1305
    - DEFLATE
    - Brotli
    - Base64
2. General interfaces for HTTP clients and servers.
   Implemented into the library:
    - [Reqwest](https://crates.io/crates/reqwest) HTTP client
    - [Axum](https://crates.io/crates/axum) HTTP server
3. REST API types implementation compatible with the protocol's paper.
4. HTTP middleware to perform and process REST API requests.
5. Port forwarding capabilities.
    - UPnP port forwarding

Author: [Nikita Podvirnyi](https://github.com/krypt0nn)\
Licensed under [AGPL-3.0](LICENSE)
