# Mecalin

A GTK4 application written in Rust using Adwaita and built with Meson.

## Quick Start

```bash
# Development
cargo run

# Production build
meson setup builddir
meson compile -C builddir
./builddir/mecalin
```

## Dependencies

- GTK4 4.10+
- libadwaita 1.5+
- Rust toolchain
- Meson build system

## Architecture

Uses GTK Builder with XML UI templates and GResource embedding for a modern GNOME application structure.
