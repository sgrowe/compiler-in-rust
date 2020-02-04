# A really hacky attempt at a compiler written in Rust

This is a WIP attempt at creating a new programming language that compiles down to [WASM](https://webassembly.org/), with the compiler written in [Rust](https://www.rust-lang.org/).

To have a look at what the syntax for the language looks like currently have a look in the [`src/fixtures`](src/fixtures) folder.

## Goals

- For me to learn more Rust and other stuff
- Tiny generated bundle size
- Greatly simplified tooling set up

## Why not compile Rust to WASM

- Higher level than Rust
- Slow Rust compile times
- Getting small bundle sizes with Rust takes a fair bit of effort
