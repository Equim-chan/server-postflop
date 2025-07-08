# Server Postflop

**Server Postflop** is a free, open-source GTO solver for Texas hold'em poker.

This is a fork of [Desktop Postflop](https://github.com/b-inary/desktop-postflop), adapted to run as an HTTP server application. This allows users to host the solver remotely and access it from lower-spec devices. Currently, this is a very primitive implementation. It has no authentication and no session management, operating with a single global state.

## Related repositories
- Solver engine: https://github.com/b-inary/postflop-solver

## Build
```shell
$ pnpm install
$ pnpm build
$ cd rust
$ cargo build --release
$ # Or if you want to use the custom-alloc feature
$ cargo +nightly build --release --features custom-alloc
```

## Usage
```shell
$ target/release/server-postflop --help
```
