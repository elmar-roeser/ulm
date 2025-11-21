# ulm - Der ULMer

AI-powered manpage assistant using local LLM inference.

## Overview

ulm transforms CLI interaction from "memorize commands" to "describe intent." It's an AI-powered bridge between what users want to accomplish and the thousands of powerful but cryptic Unix tools available on their system.

## Features

- **Discovery** - Finds the right tool even when you don't know it exists
- **Education** - Explains WHY each flag works, not just WHAT to type
- **Efficiency** - Zero context switching, all in the terminal
- **Privacy** - All processing occurs locally using Ollama

## Installation

```bash
cargo install ulm
```

### Prerequisites

- [Ollama](https://ollama.ai) installed and running
- `man` command available (man-db)

## Usage

```bash
# Initialize ulm with Ollama and index manpages
ulm setup

# Ask a question
ulm "find large files in current directory"

# Update manpage index
ulm update
```

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Run with logging
RUST_LOG=ulm=debug cargo run -- "your query"
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please follow [Conventional Commits](https://www.conventionalcommits.org/) for commit messages.
