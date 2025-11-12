# LazyJira

A terminal-based interactive interface for Jira, inspired by lazygit's intuitive and efficient workflow.

## Overview

LazyJira provides fast, keyboard-driven control over Jira tickets directly from the terminal, eliminating the need to switch between terminal and browser. Built with Rust and following strict Test-Driven Development (TDD) principles.

## Features

- 🎯 **Keyboard-first interface** - All operations accessible via keyboard shortcuts
- ⚡ **Fast and efficient** - Minimize keystrokes and context switching
- 🎨 **Visual feedback** - Clear, colorized terminal UI with status indicators
- 🔍 **Powerful search** - JQL queries and quick filters
- 🚀 **Quick actions** - Common operations with single keystrokes
- 📋 **Multiple views** - List, detail, board, and sprint views
- 🔐 **Secure** - Integration with jira-cli for authentication

## Project Status

🚧 **Early Development** - This project is in active development.

## Documentation

- [Specification](./SPECIFICATION.md) - Complete project specification
- [Architecture](./ARCHITECTURE.md) - System architecture and design
- [TDD Guidelines](./TDD_GUIDELINES.md) - Testing standards and practices
- [Features](./FEATURES.md) - Detailed feature specifications

## Requirements

- Rust 1.70+ (or latest stable)
- jira-cli (for authentication)
- Terminal with color support

## Installation

*Coming soon - project is in early development*

## Quick Start

*Coming soon*

## Keyboard Shortcuts

*Coming soon - see [Specification](./SPECIFICATION.md) for planned shortcuts*

## Configuration

*Coming soon - see [Specification](./SPECIFICATION.md) for configuration details*

## Development

### Prerequisites

- Rust toolchain
- `cargo` package manager

### Setup

```bash
# Clone the repository
git clone <repository-url>
cd lazyjira

# Run tests
cargo test

# Run with coverage
cargo tarpaulin
```

### Project Structure

```
lazyjira/
├── src/           # Source code
├── tests/         # Integration tests
├── docs/          # Documentation
└── Cargo.toml     # Dependencies
```

### Testing

This project follows strict TDD. See [TDD Guidelines](./TDD_GUIDELINES.md) for details.

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run with coverage
cargo tarpaulin --out Html
```

## Contributing

*Contributing guidelines coming soon*

## Roadmap

### Phase 1: Foundation (MVP)
- [ ] Project setup with TDD structure
- [ ] Authentication integration
- [ ] Basic ticket list view
- [ ] Ticket detail view
- [ ] Basic navigation

### Phase 2: Core Operations
- [ ] Ticket creation
- [ ] Ticket editing
- [ ] Status transitions
- [ ] Commenting
- [ ] Search and filtering

### Phase 3: Enhanced Features
- [ ] Quick actions
- [ ] Board/sprint views
- [ ] Time tracking
- [ ] Bulk operations
- [ ] Advanced filtering

### Phase 4: Polish
- [ ] Performance optimization
- [ ] UI/UX improvements
- [ ] Documentation
- [ ] Error handling refinement

See [Features](./FEATURES.md) for detailed feature specifications.

## License

*License to be determined*

## Acknowledgments

- Inspired by [lazygit](https://github.com/jesseduffield/lazygit)
- Integrates with [jira-cli](https://github.com/ankitpokhrel/jira-cli)

## Related Projects

- [lazygit](https://github.com/jesseduffield/lazygit) - Simple terminal UI for git commands
- [jira-cli](https://github.com/ankitpokhrel/jira-cli) - Feature-rich interactive Jira command line tool
