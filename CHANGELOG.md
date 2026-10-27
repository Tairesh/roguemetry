# Changelog

All notable changes to this project will be documented in this file.
See [Keep a Changelog](https://keepachangelog.com/) for details.
This project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.4.1] - 2026-10-28

### Changed

- Update locked dependencies (`cargo update`).

### Fixed

- `just check` passes on clippy 1.99, which adds the pedantic `assert_is_empty` lint.

## [0.4.0] - 2026-10-28

### Added

- `GridSize` is its own type: `w()`/`h()`, `width()`/`height()` as coordinates, `stride()`, `area()`,
  `contains`, `index_of`, `point_at`, `tiles()`, `rows()`, and the index neighbours
  `north_of`/`east_of`/`south_of`/`west_of`. `GridSize::new` checks that each side is at most
  `i32::MAX` and the area at most `isize::MAX`; `GridSize::try_from((w, h))` returns `GridSizeError`
  instead of panicking. With `serde`, it reads `{ "w", "h" }` or `[w, h]`.
- `Direction::rotated(eighths)`, `rotate_cw()`, `rotate_ccw()`.
- `Direction::angle()`, `Direction::from_angle(θ)`, `Direction::nearest(v)`: y down, North = 0,
  clockwise; exact halfway ties go clockwise.
- `Direction * n` gives the `Point` offset of `n` steps.

### Changed

- **Breaking:** `DIR8` runs clockwise from North; `DIR9` is `Here` then `DIR8`. `Direction::random`
  draws from them, so a seed yields different directions.
- **Breaking:** `field_of_view` and `Fov::compute` take a `GridSize`; a `(w, h)` tuple no longer
  converts into one. `Fov::size()` returns `GridSize`.
- `field_of_view` no longer panics on any origin, and its scan no longer recurses, so a long range
  cannot overflow the stack.

### Removed

- **Breaking:** `Point::to_index` and `Point::from_index` — use `GridSize::index_of` and
  `GridSize::point_at`.
- **Breaking:** the `GridSize` alias of `vek::Extent2<u32>` and the deprecated `Extent2` alias.

## [0.3.1] - 2026-10-27

### Changed

- Rename the `vek::Extent2<u32>` alias to `GridSize`.

### Deprecated

- `Extent2` stays as an alias of `GridSize`, so 0.3.0 code still compiles. Switch to `GridSize`.

## [0.3.0] - 2026-10-27

### Added

- `field_of_view(origin, range, size, is_opaque)` returns a dense `Fov` grid.
  It ports the recursive shadowcasting from `bracket-pathfinding` and matches its `field_of_view_set` cell for cell.
- `Fov::compute` refills an existing `Fov` and reuses its buffer.
- `Extent2` alias for `vek::Extent2<u32>`, used for FOV sizes.

### Changed

- **Breaking:** `circle` takes a `u32` radius.
- **Breaking:** `Point::to_index` takes a `u32` width.
- **Breaking:** `Point::from_index` takes a `NonZeroU32` width.

### Fixed

- `Point::to_index`, `Point::from_index` and `Point::square_distance_to` no longer overflow on large coordinates.
  `square_distance_to` saturates at `u32::MAX`.

## [0.2.1] - 2026-08-31

### Added

- `DIR4`, `DIR4_OPPOSITE` and `DIR2_EDGE_ONCE` constants.
- `Direction::orthogonals`, `Direction::opposite` and `Direction::is_orthogonal`.

## [0.2.0] - 2026-03-11

### Changed

- **Breaking:** upgrade `rand` to 0.10. `Direction::random` and `Point::random` require `rand::RngExt` on the generator.

## [0.1.1] - 2025-09-28

### Changed

- **Breaking:** rename `TwoDimDirection` to `OneDimensionalDirection` and `ConvertError` to `OneDimensionConvertError`.
- Upgrade `vek` to 0.17.

## [0.1.0] - 2025-07-21

### Added

- `Point` with `i32` coordinates, operators for `Direction`, tuples, `Vec2` and scalars, map index conversion, distance and Bresenham lines.
- `Direction` enum with nine variants, `DIR8` and `DIR9` constants, and conversions from deltas, points and tuples.
- `TwoDimDirection` for East/West movement.
- Precomputed circles `CIRCLE5` to `CIRCLE13` and a `circle(center, radius)` function.
- Code Page 437 conversion between Unicode characters and bytes.
- `Vec2` and `Rect` aliases over `vek` types.
- Optional `rand` and `serde` features, both on by default.

[Unreleased]: https://github.com/Tairesh/roguemetry/compare/v0.4.1...HEAD
[0.4.1]: https://github.com/Tairesh/roguemetry/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/Tairesh/roguemetry/compare/v0.3.1...v0.4.0
[0.3.1]: https://github.com/Tairesh/roguemetry/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/Tairesh/roguemetry/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/Tairesh/roguemetry/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/Tairesh/roguemetry/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/Tairesh/roguemetry/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/Tairesh/roguemetry/releases/tag/v0.1.0
