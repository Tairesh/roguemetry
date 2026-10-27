// Ported from bracket-pathfinding (https://github.com/thebracket/bracket-lib), MIT License:
//
// Copyright 2019 Herbert Wolverson (DBA Bracket Productions)
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of this software
// and associated documentation files (the "Software"), to deal in the Software without
// restriction, including without limitation the rights to use, copy, modify, merge, publish,
// distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all copies or
// substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING
// BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
// NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
// DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

//! Field of view by recursive shadowcasting.
//!
//! The algorithm is a cell-for-cell port of `bracket-pathfinding` 0.8.7's `field_of_view_set`
//! (<https://github.com/thebracket/bracket-lib>): the same eight 45° rays, the same eight octant
//! scans and the same `f32` slope arithmetic. The result is a dense grid instead of a hash set.

use super::{Extent2, Point};

/// The eight 45° rays cast before the octant scans: N, NE, E, SE, S, SW, W, NW.
const SECTORS: [(i32, i32); 8] = [
    (0, -1),
    (1, -1),
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
];

/// Cells visible from one origin. Reuse with [`Fov::compute`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fov {
    size: Point,
    visible: Vec<bool>,
    /// Inclusive lower corner of the cells marked visible; `min > max` when nothing is.
    min: Point,
    /// Inclusive upper corner of the cells marked visible.
    max: Point,
}

impl Default for Fov {
    fn default() -> Self {
        Self::new()
    }
}

impl Fov {
    #[must_use]
    /// Create an empty field of view over a zero-sized rectangle.
    pub fn new() -> Self {
        Self {
            size: Point::new(0, 0),
            visible: Vec::new(),
            min: Point::new(0, 0),
            max: Point::new(-1, -1),
        }
    }

    #[must_use]
    /// The rectangle the field of view was computed over.
    pub fn size(&self) -> Extent2 {
        Extent2::new(self.size.x.unsigned_abs(), self.size.y.unsigned_abs())
    }

    #[must_use]
    /// Whether `point` is visible. Always `false` outside the rectangle.
    pub fn contains(&self, point: Point) -> bool {
        self.index(point).is_some_and(|index| self.visible[index])
    }

    /// The visible cells, in row-major order.
    pub fn iter(&self) -> impl Iterator<Item = Point> + '_ {
        (self.min.y..=self.max.y).flat_map(move |y| {
            (self.min.x..=self.max.x)
                .map(move |x| Point::new(x, y))
                .filter(move |&point| self.contains(point))
        })
    }

    /// [`field_of_view`] that reuses this `Fov`'s allocation.
    pub fn compute(
        &mut self,
        origin: Point,
        range: u32,
        size: impl Into<Extent2>,
        is_opaque: impl FnMut(Point) -> bool,
    ) {
        // Cells past `i32::MAX` have no `Point`, so the rectangle is clamped to what is addressable.
        let size = size
            .into()
            .map(|side| i32::try_from(side).unwrap_or(i32::MAX));
        self.reset(Point::new(size.w, size.h));
        let range_2 = range.saturating_pow(2);
        let mut scanner = Scanner {
            center: origin,
            range_2,
            is_opaque,
            fov: self,
        };
        scanner.fov.insert(origin);
        let visibility_per_sector = scanner.cast_rays();
        for (sector, octants) in [
            (0, [N2NW, N2NE]),
            (2, [E2NE, E2SE]),
            (4, [S2SE, S2SW]),
            (6, [W2SW, W2NW]),
        ] {
            if visibility_per_sector[sector] {
                for octant in octants {
                    scanner.scan(octant, 1, 0., 1.);
                }
            }
        }
    }

    /// Resize to `size` (non-negative) and clear every cell.
    #[allow(clippy::cast_sign_loss)]
    fn reset(&mut self, size: Point) {
        if size == self.size {
            for y in self.min.y..=self.max.y {
                if let (Some(start), Some(end)) = (
                    self.index(Point::new(self.min.x, y)),
                    self.index(Point::new(self.max.x, y)),
                ) {
                    self.visible[start..=end].fill(false);
                }
            }
        } else {
            self.size = size;
            self.visible.clear();
            self.visible
                .resize(size.x as usize * size.y as usize, false);
        }
        self.min = size;
        self.max = Point::new(-1, -1);
    }

    fn index(&self, point: Point) -> Option<usize> {
        if point.y < self.size.y {
            point.to_index(self.size.x.unsigned_abs())
        } else {
            None
        }
    }

    fn in_bounds(&self, point: Point) -> bool {
        point.x >= 0 && point.x < self.size.x && point.y >= 0 && point.y < self.size.y
    }

    /// Mark `point` visible; a point outside the rectangle is dropped.
    fn insert(&mut self, point: Point) {
        if let Some(index) = self.index(point) {
            self.visible[index] = true;
            self.min = Point::new(self.min.x.min(point.x), self.min.y.min(point.y));
            self.max = Point::new(self.max.x.max(point.x), self.max.y.max(point.y));
        }
    }
}

/// Cells visible from `origin` within `range`, over the rectangle `[0, size.w) × [0, size.h)`.
///
/// `is_opaque` is asked only about cells inside the rectangle. `origin` is visible even when
/// opaque.
#[must_use]
pub fn field_of_view(
    origin: Point,
    range: u32,
    size: impl Into<Extent2>,
    is_opaque: impl FnMut(Point) -> bool,
) -> Fov {
    let mut fov = Fov::new();
    fov.compute(origin, range, size, is_opaque);
    fov
}

/// One octant scan: `d` is the distance along `primary` from the origin, `t` the offset along
/// `secondary` (growing with the slope). A scan walks the line `d` in the order `t` grows.
#[derive(Clone, Copy)]
struct Octant {
    /// `true` when `d` moves along y (the N/S scans), `false` when along x (the E/W scans).
    vertical: bool,
    /// Sign of the step along the `d` axis.
    primary: i32,
    /// Sign of the step along the `t` axis.
    secondary: i32,
}

const N2NE: Octant = Octant::new(true, -1, 1);
const N2NW: Octant = Octant::new(true, -1, -1);
const S2SE: Octant = Octant::new(true, 1, 1);
const S2SW: Octant = Octant::new(true, 1, -1);
const E2SE: Octant = Octant::new(false, 1, 1);
const E2NE: Octant = Octant::new(false, 1, -1);
const W2SW: Octant = Octant::new(false, -1, 1);
const W2NW: Octant = Octant::new(false, -1, -1);

impl Octant {
    const fn new(vertical: bool, primary: i32, secondary: i32) -> Self {
        Self {
            vertical,
            primary,
            secondary,
        }
    }

    fn point(self, center: Point, d: i32, t: i32) -> Point {
        let along = d * self.primary;
        let across = t * self.secondary;
        if self.vertical {
            Point::new(center.x + across, center.y + along)
        } else {
            Point::new(center.x + along, center.y + across)
        }
    }

    /// `(primary, secondary)` components of `p` (center coordinate and rectangle extent).
    fn split(self, p: Point) -> (i32, i32) {
        if self.vertical {
            (p.y, p.x)
        } else {
            (p.x, p.y)
        }
    }
}

/// Whether coordinate `c` has left `[0, extent)` on the side `sign` walks toward.
fn beyond(c: i32, sign: i32, extent: i32) -> bool {
    if sign > 0 {
        c >= extent
    } else {
        c < 0
    }
}

/// Largest offset `t` along `sign` from `center` that stays inside `[0, extent)` on that side.
fn last_offset(center: i32, sign: i32, extent: i32) -> i32 {
    if sign > 0 {
        extent - 1 - center
    } else {
        center
    }
}

struct Scanner<'a, F> {
    center: Point,
    range_2: u32,
    is_opaque: F,
    fov: &'a mut Fov,
}

impl<F: FnMut(Point) -> bool> Scanner<'_, F> {
    fn is_transparent(&mut self, point: Point) -> bool {
        self.fov.in_bounds(point) && !(self.is_opaque)(point)
    }

    #[allow(clippy::cast_precision_loss)]
    fn distance_to_center(&self, point: Point) -> f32 {
        let dx = i32::abs(point.x - self.center.x) as f32 - 0.5;
        let dy = i32::abs(point.y - self.center.y) as f32 - 0.5;
        dx * dx + dy * dy
    }

    /// The 45° pre-pass: walk each ray, marking cells until one is opaque or out of range.
    /// Returns, per sector, whether the ray saw past the origin's neighbour.
    fn cast_rays(&mut self) -> [bool; 8] {
        let mut visibility_per_sector = [false; 8];
        for (visible, (dx, dy)) in visibility_per_sector.iter_mut().zip(SECTORS) {
            let mut current = self.center;
            loop {
                current = Point::new(current.x + dx, current.y + dy);
                if !self.fov.in_bounds(current) {
                    break;
                }
                if current.square_distance_to(self.center) > self.range_2 {
                    break;
                }
                self.fov.insert(current);
                if (self.is_opaque)(current) {
                    break;
                }
                *visible = true;
            }
        }
        visibility_per_sector
    }

    /// Mark `point` if within range (off the scan's own axis, `t != 0`); return whether light
    /// passes through it.
    #[allow(clippy::cast_precision_loss)]
    fn insert_visible(&mut self, point: Point, t: i32) -> bool {
        if self.distance_to_center(point) > self.range_2 as f32 {
            return false;
        }
        if t != 0 {
            self.fov.insert(point);
        }
        self.is_transparent(point)
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    fn scan(&mut self, octant: Octant, distance: i32, start_slope: f32, end_slope: f32) {
        let mut start_slope = start_slope;

        if distance.unsigned_abs().saturating_pow(2) > self.range_2 {
            return;
        }

        let (center_d, center_t) = octant.split(self.center);
        let (extent_d, extent_t) = octant.split(self.fov.size);
        if beyond(
            center_d + octant.primary * distance,
            octant.primary,
            extent_d,
        ) {
            return;
        }

        let start_t = (start_slope * distance as f32 + 0.5) as i32;
        if beyond(
            center_t + octant.secondary * start_t,
            octant.secondary,
            extent_t,
        ) {
            return;
        }

        let end_t = ((end_slope * distance as f32 + 0.5) as i32).min(last_offset(
            center_t,
            octant.secondary,
            extent_t,
        ));

        let mut last_visible = self.is_transparent(octant.point(self.center, distance, start_t));
        for t in start_t..=end_t {
            let current = octant.point(self.center, distance, t);
            let is_visible = self.insert_visible(current, t);

            if last_visible && !is_visible {
                self.scan(
                    octant,
                    distance + 1,
                    start_slope,
                    (t as f32 - 0.5) / (distance as f32 + 0.5),
                );
            } else if !last_visible && is_visible {
                start_slope = (t as f32 - 0.5) / (distance as f32 - 0.5);
            }
            last_visible = is_visible;
        }
        if last_visible {
            self.scan(octant, distance + 1, start_slope, end_slope);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{field_of_view, Extent2, Fov, Point};
    use crate::circles::CIRCLE5;
    use bracket_pathfinding::prelude::{
        field_of_view_set, Algorithm2D, BaseMap, Point as BracketPoint,
    };
    use rand::rngs::Xoshiro256PlusPlus;
    use rand::{RngExt, SeedableRng};

    /// An opacity grid, shared by our cast and bracket's.
    struct Grid {
        size: Point,
        opaque: Vec<bool>,
    }

    impl Grid {
        fn open(size: Point) -> Self {
            Self::filled(size, |_| false)
        }

        fn filled(size: Point, mut opaque: impl FnMut(Point) -> bool) -> Self {
            let mut cells = Vec::new();
            for y in 0..size.y.max(0) {
                for x in 0..size.x.max(0) {
                    cells.push(opaque(Point::new(x, y)));
                }
            }
            Self {
                size,
                opaque: cells,
            }
        }

        fn is_opaque(&self, point: Point) -> bool {
            assert!(
                point.x >= 0 && point.x < self.size.x && point.y >= 0 && point.y < self.size.y,
                "is_opaque asked about {point:?} outside {:?}",
                self.size
            );
            self.opaque[usize::try_from(point.y * self.size.x + point.x).unwrap()]
        }

        fn extent(&self) -> Extent2 {
            Extent2::new(self.size.x.unsigned_abs(), self.size.y.unsigned_abs())
        }

        fn fov(&self, origin: Point, range: u32) -> Fov {
            field_of_view(origin, range, self.extent(), |p| self.is_opaque(p))
        }

        fn bracket(&self, origin: Point, range: u32) -> Vec<Point> {
            let range = i32::try_from(range).unwrap();
            let mut cells: Vec<Point> =
                field_of_view_set(BracketPoint::new(origin.x, origin.y), range, self)
                    .into_iter()
                    .map(|p| Point::new(p.x, p.y))
                    .collect();
            cells.sort_by_key(|p| (p.y, p.x));
            cells
        }
    }

    impl BaseMap for Grid {
        fn is_opaque(&self, idx: usize) -> bool {
            self.opaque[idx]
        }
    }

    impl Algorithm2D for Grid {
        fn dimensions(&self) -> BracketPoint {
            BracketPoint::new(self.size.x, self.size.y)
        }

        // Bracket only uses the index of an in-bounds cell, but computes one for an
        // out-of-bounds cell too when the origin lies outside the map; the default panics on a
        // negative index there.
        fn point2d_to_index(&self, pt: BracketPoint) -> usize {
            if self.in_bounds(pt) {
                usize::try_from(pt.y * self.size.x + pt.x).unwrap()
            } else {
                0
            }
        }
    }

    fn origins(rng: &mut Xoshiro256PlusPlus, size: Point) -> Vec<Point> {
        let (w, h) = (size.x.max(1), size.y.max(1));
        let (right, bottom) = (w - 1, h - 1);
        let mut origins = vec![
            Point::new(0, 0),
            Point::new(right, 0),
            Point::new(0, bottom),
            Point::new(right, bottom),
            Point::new(w / 2, 0),
            Point::new(w / 2, bottom),
            Point::new(0, h / 2),
            Point::new(right, h / 2),
            Point::new(rng.random_range(0..w), 0),
            Point::new(rng.random_range(0..w), bottom),
            Point::new(0, rng.random_range(0..h)),
            Point::new(right, rng.random_range(0..h)),
            Point::new(w / 2, h / 2),
            Point::new(-1, h / 2),
            Point::new(w + 2, h + 3),
        ];
        for _ in 0..4 {
            origins.push(Point::new(rng.random_range(0..w), rng.random_range(0..h)));
        }
        origins
    }

    #[test]
    fn matches_bracket_on_random_maps() {
        const SIZES: [(i32, i32); 9] = [
            (0, 0),
            (1, 1),
            (1, 17),
            (17, 1),
            (2, 2),
            (5, 3),
            (13, 13),
            (40, 21),
            (120, 90),
        ];
        const DENSITIES: [f64; 4] = [0.0, 0.1, 0.4, 1.0];
        const RANGES: [u32; 10] = [0, 1, 2, 3, 5, 8, 13, 30, 64, 200];

        let mut rng = Xoshiro256PlusPlus::seed_from_u64(0x00F0_5EED);
        let mut reused = Fov::new();
        let mut cases = 0;
        for (w, h) in SIZES {
            let size = Point::new(w, h);
            for density in DENSITIES {
                for _ in 0..3 {
                    let grid = Grid::filled(size, |_| rng.random_bool(density));
                    for origin in origins(&mut rng, size) {
                        for range in RANGES {
                            reused.compute(origin, range, grid.extent(), |p| grid.is_opaque(p));
                            let ours: Vec<Point> = reused.iter().collect();
                            assert_eq!(
                                ours,
                                grid.bracket(origin, range),
                                "size {size:?}, density {density}, origin {origin:?}, range {range}"
                            );
                            assert_eq!(reused, grid.fov(origin, range));
                            cases += 1;
                        }
                    }
                }
            }
        }
        assert_eq!(cases, SIZES.len() * DENSITIES.len() * 3 * 19 * RANGES.len());
    }

    #[test]
    fn open_room_is_a_disc() {
        let grid = Grid::open(Point::new(7, 7));
        let center = Point::new(3, 3);
        let mut expected: Vec<Point> = CIRCLE5
            .iter()
            .map(|&(dx, dy)| Point::new(center.x + dx, center.y + dy))
            .collect();
        expected.sort_by_key(|p| (p.y, p.x));
        let fov = grid.fov(center, 2);
        assert_eq!(fov.iter().collect::<Vec<_>>(), expected);
    }

    #[test]
    fn whole_open_room_in_long_range() {
        let grid = Grid::open(Point::new(9, 6));
        let fov = grid.fov(Point::new(4, 2), 50);
        assert_eq!(fov.iter().count(), 9 * 6);
    }

    #[test]
    fn wall_casts_a_shadow() {
        let wall = Point::new(3, 1);
        let grid = Grid::filled(Point::new(9, 3), |p| p == wall);
        let fov = grid.fov(Point::new(1, 1), 20);
        assert!(fov.contains(wall));
        for x in 4..9 {
            assert!(!fov.contains(Point::new(x, 1)), "({x}, 1) behind the wall");
        }
        assert!(fov.contains(Point::new(4, 0)));
        assert!(fov.contains(Point::new(4, 2)));
        assert!(!fov.contains(Point::new(8, 0)), "the shadow widens");
    }

    #[test]
    fn origin_inside_solid_rock_sees_its_neighbours() {
        let grid = Grid::filled(Point::new(5, 5), |_| true);
        let fov = grid.fov(Point::new(2, 2), 5);
        let expected: Vec<Point> = (1..=3)
            .flat_map(|y| (1..=3).map(move |x| Point::new(x, y)))
            .collect();
        assert_eq!(fov.iter().collect::<Vec<_>>(), expected);
    }

    #[test]
    fn opaque_origin_does_not_block_its_own_sight() {
        let origin = Point::new(4, 3);
        let open = Grid::open(Point::new(9, 7));
        let rock = Grid::filled(Point::new(9, 7), |p| p == origin);
        assert_eq!(rock.fov(origin, 6), open.fov(origin, 6));
    }

    #[test]
    fn range_zero_is_the_origin() {
        let grid = Grid::open(Point::new(5, 5));
        let fov = grid.fov(Point::new(1, 2), 0);
        assert_eq!(fov.iter().collect::<Vec<_>>(), vec![Point::new(1, 2)]);
    }

    #[test]
    fn origin_outside_is_not_visible() {
        let grid = Grid::open(Point::new(5, 5));
        let origin = Point::new(-1, 2);
        let fov = grid.fov(origin, 3);
        assert!(!fov.contains(origin));
        assert!(fov.contains(Point::new(0, 2)));
    }

    #[test]
    fn contains_is_false_outside_the_rectangle() {
        let grid = Grid::open(Point::new(3, 3));
        let fov = grid.fov(Point::new(1, 1), 10);
        assert_eq!(fov.size(), Extent2::new(3, 3));
        for p in [
            Point::new(-1, 0),
            Point::new(0, -1),
            Point::new(3, 0),
            Point::new(0, 3),
        ] {
            assert!(!fov.contains(p));
        }
    }

    #[test]
    fn size_accepts_a_tuple() {
        let fov = field_of_view(Point::new(1, 1), 1, (4, 3), |_| false);
        assert_eq!(fov.size(), Extent2::new(4, 3));
    }

    #[test]
    fn default_is_empty() {
        let fov = Fov::default();
        assert_eq!(fov.size(), Extent2::new(0, 0));
        assert_eq!(fov.iter().count(), 0);
        assert!(!fov.contains(Point::new(0, 0)));
    }
}
