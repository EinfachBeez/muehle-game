# Nine Men's Morris (Mühle) in Rust

A complete implementation of **Nine Men's Morris** (Mühle) in Rust, featuring a graphical UI, AI agent and various search algorithms.
Built as a programming project at the University of Stuttgart, this was my first project in Rust.

## Build

Build the game using Cargo:

```bash
cargo build --release 
```

Build individual binaries (ai, search algorithms):

```bash
cargo build --release --bin complete_search
cargo build --release --bin ai_agent
```

## Usage

Run the game:

```bash
cargo run --release
```

Or execute various binaries:

```bash
cargo run --bin complete_search -- input.txt output.txt
```

## Test

```
cargo test
```