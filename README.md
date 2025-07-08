# Server Postflop

**Server Postflop** is a free, open-source GTO solver for Texas hold'em poker.

This is a fork of [Desktop Postflop](https://github.com/b-inary/desktop-postflop), adapted to run as an HTTP server application. This allows users to host the solver remotely and access it from lower-spec devices. Currently, This is a very basic implementation. It has no authorization and no session management, operating with a single global session.

## Related repositories
- Solver engine: https://github.com/b-inary/postflop-solver

## Build
```shell
$ pnpm install
$ pnpm build
$ cd rust
$ cargo build --release
```

## Usage
```shell
$ target/release/server-postflop --help
```
