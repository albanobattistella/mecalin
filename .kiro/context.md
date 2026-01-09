# Mecalin Project Context

## Project Overview
Mecalin is a typing tutor application built with GTK4, Rust, and Adwaita. It follows modern GNOME application patterns with GTK Builder XML UI templates and GResource embedding.

## Technology Stack
- **Language**: Rust
- **UI Framework**: GTK4 4.10+
- **Design**: libadwaita 1.5+ (Adwaita design system)
- **Build System**: Meson (production), Cargo (development)
- **Architecture**: GTK Builder with XML UI templates

## Development Workflow

### Quick Commands
```bash
# Development/testing
cargo run

# Production build
meson setup builddir
meson compile -C builddir
./builddir/mecalin
```

### Pre-commit Requirements
- **ALWAYS run `cargo fmt` before committing**
- Run `cargo clippy` to check for warnings
- Ensure code compiles without errors

### Code Conventions
- Follow Rust standard conventions (rustfmt, clippy)
- Use GTK4/Adwaita patterns for UI components
- Embed UI resources using GResource
- Separate UI templates (XML) from logic (Rust)

## Project Structure
- UI templates should be in XML format for GTK Builder
- Follow GNOME application structure guidelines
- Use Adwaita design patterns for consistent UX

## Dependencies Management
- Core dependencies: GTK4, libadwaita, Rust toolchain, Meson
- Keep dependencies minimal and well-justified
- Prefer stable, well-maintained crates

## AI Assistant Guidelines
- Prioritize GTK4/Adwaita best practices
- Suggest modern Rust patterns appropriate for GUI applications
- Consider both development (Cargo) and production (Meson) build workflows
- Focus on GNOME HIG compliance for UI suggestions
- Keep code concise and maintainable
- Always remind about running `cargo fmt` before commits
