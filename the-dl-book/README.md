# The Rust Download Book

A book about building an over-engineered download manager in Rust, complete with custom caricature sprites for admonitions.

## Features

- **12 Unique Admonition Types** - All built-in mdbook-admonish types with unique caricature sprites
- **Theme-Aware** - Works seamlessly across all mdbook themes (Light, Rust, Coal, Navy, Ayu)
- **Web-Optimized** - All sprites optimized to ~18KB each for fast loading
- **Clean Syntax** - Uses ` ```admonish type` markdown blocks (no HTML required)

## Quick Start

Build the book:
```bash
mdbook build
```

Serve locally:
```bash
mdbook serve
```

## Admonition System

This book uses mdbook-admonish with custom caricature sprites. See [ADMONITIONS.md](./ADMONITIONS.md) for a complete reference with visual examples of all 12 admonition types.

### All 12 Types
Use ` ```admonish type` syntax for: note, abstract, info, tip, success, question, warning, failure, danger, bug, example, quote

Each type has aliases (e.g., `tip`/`hint`/`important` all show the same sprite) and supports custom titles and collapsible sections.

## Project Structure

```
the-dl-book/
├── src/              # Book source files
│   ├── sprites/      # Caricature sprites (120x120px)
│   └── *.md          # Chapter content
├── theme/            # Custom CSS and assets
│   └── custom.css    # Admonition styling
├── book.toml         # mdbook configuration
└── ADMONITIONS.md    # Complete admonition reference
```

## Meet Mack

Mack is a crocodile mascot who guides you through the book with 12 unique expressions. Visit the [Meet Mack](./src/meet_mack.md) page in the book to see all admonition types in action!
