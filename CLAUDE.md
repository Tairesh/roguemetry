# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Roguemetry is a Rust utility library for roguelike game development. It provides 2D point math, direction enums, circle generation, and CP437 character encoding. Published to crates.io.

## Build & Development Commands

```bash
just check          # Run fmt-check, test, and clippy (used in CI)
just test           # cargo test
just clippy         # cargo clippy -- -D warnings -D clippy::pedantic --verbose --no-deps
just fmt            # cargo fmt
just before-commit  # fmt, update, check — run before committing
just build          # cargo build --release
```

Run a single test: `cargo test <test_name>`

## Clippy Configuration

Clippy runs with `-D warnings -D clippy::pedantic`. All warnings are errors. Fix all clippy pedantic lints before committing.

## Feature Flags

Default features: `rand`, `serde` (both optional). Code gated behind these features uses `#[cfg(feature = "...")]`.

## Architecture

- `src/lib.rs` — Library root, re-exports all modules. Defines `Vec2` (`vek::Vec2<f32>`) and `Rect` (`vek::Rect<f32, f32>`) type aliases.
- `src/point.rs` — `Point` struct (i32 x/y). Extensive operator overloading with Direction, tuples, Vec2, scalars. Map index conversion (`to_index`/`from_index`), distance, line drawing (Bresenham).
- `src/direction.rs` — `Direction` enum (9 variants including Here). `DIR8`/`DIR9` constants. Converts from deltas, Points, tuples.
- `src/one_direction.rs` — `OneDimensionalDirection` (East/West only). TryFrom<Direction> with `OneDimensionConvertError`.
- `src/circles.rs` — Pre-calculated circle point arrays (CIRCLE5–CIRCLE13) and dynamic `circle(center, radius)` function.
- `src/cp437.rs` — Code Page 437 bidirectional mapping (Unicode ↔ CP437 bytes). Lookup table based.

## Conventions

- Tests are inline `#[cfg(test)]` modules within each source file
- Commit messages use gitmoji prefixes (e.g., `:fire:`, `:memo:`, `:recycle:`)
