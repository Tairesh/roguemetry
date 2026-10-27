//! Grid dimensions and row-major tile indexing.

use std::fmt;
use std::num::NonZeroUsize;
use std::ops::Range;

use super::Point;

/// A grid's width × height in tiles, indexed row-major.
///
/// Each side is at most `i32::MAX`, so every tile has a [`Point`], and the area is at most
/// `isize::MAX`, so every index fits `usize`. A zero side is allowed: the grid is then empty.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "Sides", into = "Sides"))]
pub struct GridSize {
    w: u32,
    h: u32,
}

/// Sides that [`GridSize::try_from`] rejects: see [`GridSize::new`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct GridSizeError;

impl fmt::Display for GridSizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("grid side exceeds i32::MAX or area exceeds isize::MAX")
    }
}

impl std::error::Error for GridSizeError {}

const fn fits(w: u32, h: u32) -> bool {
    let max_side = i32::MAX.unsigned_abs();
    if w > max_side || h > max_side {
        return false;
    }
    match (w as usize).checked_mul(h as usize) {
        Some(area) => area <= isize::MAX.unsigned_abs(),
        None => false,
    }
}

impl GridSize {
    /// # Panics
    ///
    /// Panics if a side exceeds `i32::MAX` or the area exceeds `isize::MAX`.
    #[inline]
    #[must_use]
    pub const fn new(w: u32, h: u32) -> Self {
        assert!(
            fits(w, h),
            "grid side exceeds i32::MAX or area exceeds isize::MAX"
        );
        Self { w, h }
    }

    #[inline]
    #[must_use]
    pub const fn w(self) -> u32 {
        self.w
    }

    #[inline]
    #[must_use]
    pub const fn h(self) -> u32 {
        self.h
    }

    /// The width as a coordinate: the first column past the grid.
    #[inline]
    #[must_use]
    #[allow(clippy::cast_possible_wrap)] // w ≤ i32::MAX
    pub const fn width(self) -> i32 {
        self.w as i32
    }

    /// The height as a coordinate: the first row past the grid.
    #[inline]
    #[must_use]
    #[allow(clippy::cast_possible_wrap)] // h ≤ i32::MAX
    pub const fn height(self) -> i32 {
        self.h as i32
    }

    /// Index distance between vertically adjacent tiles.
    #[inline]
    #[must_use]
    pub const fn stride(self) -> usize {
        self.w as usize
    }

    #[inline]
    #[must_use]
    pub const fn area(self) -> usize {
        self.w as usize * self.h as usize
    }

    #[inline]
    #[must_use]
    #[allow(clippy::cast_sign_loss)] // a negative coordinate wraps past i32::MAX and fails
    pub const fn contains(self, p: Point) -> bool {
        (p.x as u32) < self.w && (p.y as u32) < self.h
    }

    /// Row-major index of `p`; `None` off the grid.
    #[inline]
    #[must_use]
    #[allow(clippy::cast_sign_loss)] // non-negative once `contains` holds
    pub fn index_of(self, p: Point) -> Option<usize> {
        if self.contains(p) {
            Some(p.y as u32 as usize * self.stride() + p.x as u32 as usize)
        } else {
            None
        }
    }

    /// The tile at row-major `index`; `None` past [`GridSize::area`].
    #[inline]
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)] // x < w, y < h
    pub fn point_at(self, index: usize) -> Option<Point> {
        if index >= self.area() {
            return None;
        }
        let w = NonZeroUsize::new(self.stride())?;
        Some(Point::new((index % w) as i32, (index / w) as i32))
    }

    /// Every tile, row-major: `tiles().enumerate()` yields `(index_of(p), p)`.
    #[inline]
    pub fn tiles(self) -> impl Iterator<Item = Point> {
        let (w, h) = (self.width(), self.height());
        (0..h).flat_map(move |y| (0..w).map(move |x| Point::new(x, y)))
    }

    /// Each row's index range, top to bottom. Walk neighbours by index inside a row to avoid a
    /// division per tile.
    #[inline]
    pub fn rows(self) -> impl Iterator<Item = Range<usize>> {
        let w = self.stride();
        (0..self.h as usize).map(move |y| y * w..y * w + w)
    }

    /// Index of the tile above `i`; `None` if either is off the grid.
    #[inline]
    #[must_use]
    pub fn north_of(self, i: usize) -> Option<usize> {
        let s = self.stride();
        (i.wrapping_sub(s) < self.area().saturating_sub(s)).then(|| i - s)
    }

    /// Index of the tile below `i`; `None` if either is off the grid.
    #[inline]
    #[must_use]
    pub fn south_of(self, i: usize) -> Option<usize> {
        let s = self.stride();
        (i < self.area().saturating_sub(s)).then(|| i + s)
    }

    /// Index of the tile right of `i`; `None` if either is off the grid. Divides; see
    /// [`GridSize::rows`] for hot loops.
    #[inline]
    #[must_use]
    pub fn east_of(self, i: usize) -> Option<usize> {
        if i >= self.area() {
            return None;
        }
        let w = NonZeroUsize::new(self.stride())?;
        (i % w + 1 < w.get()).then(|| i + 1)
    }

    /// Index of the tile left of `i`; `None` if either is off the grid. Divides; see
    /// [`GridSize::rows`] for hot loops.
    #[inline]
    #[must_use]
    pub fn west_of(self, i: usize) -> Option<usize> {
        if i >= self.area() {
            return None;
        }
        let w = NonZeroUsize::new(self.stride())?;
        (i % w != 0).then(|| i - 1)
    }
}

impl TryFrom<(u32, u32)> for GridSize {
    type Error = GridSizeError;

    #[inline]
    fn try_from((w, h): (u32, u32)) -> Result<Self, Self::Error> {
        if fits(w, h) {
            Ok(Self { w, h })
        } else {
            Err(GridSizeError)
        }
    }
}

/// The serde shape of a [`GridSize`]: `{ w, h }`, or `[w, h]` when deserializing.
#[cfg(feature = "serde")]
#[derive(serde::Serialize, serde::Deserialize)]
struct Sides {
    w: u32,
    h: u32,
}

#[cfg(feature = "serde")]
impl TryFrom<Sides> for GridSize {
    type Error = GridSizeError;

    fn try_from(sides: Sides) -> Result<Self, Self::Error> {
        Self::try_from((sides.w, sides.h))
    }
}

#[cfg(feature = "serde")]
impl From<GridSize> for Sides {
    fn from(size: GridSize) -> Self {
        Self {
            w: size.w,
            h: size.h,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{GridSize, GridSizeError};
    use crate::{Point, DIR4};

    #[test]
    fn index_and_point_round_trip_over_every_tile() {
        let size = GridSize::new(5, 3);
        let tiles: Vec<Point> = size.tiles().collect();
        assert_eq!(tiles.len(), size.area());
        for (i, &p) in tiles.iter().enumerate() {
            assert!(size.contains(p));
            assert_eq!(size.index_of(p), Some(i));
            assert_eq!(size.point_at(i), Some(p));
        }
        for (i, p) in size.tiles().enumerate() {
            assert_eq!(
                size.index_of(p),
                Some(i),
                "tiles().enumerate() pairs index and point"
            );
        }
    }

    #[test]
    fn getters() {
        let size = GridSize::new(5, 3);
        assert_eq!((size.w(), size.h()), (5, 3));
        assert_eq!((size.width(), size.height()), (5, 3));
        assert_eq!((size.stride(), size.area()), (5, 15));
    }

    #[test]
    fn tiles_off_the_grid_have_no_index() {
        let size = GridSize::new(5, 3);
        for p in [
            Point::new(-1, 0),
            Point::new(0, -1),
            Point::new(5, 0),
            Point::new(0, 3),
            Point::new(i32::MIN, 0),
            Point::new(0, i32::MIN),
            Point::new(i32::MAX, 0),
            Point::new(0, i32::MAX),
        ] {
            assert!(!size.contains(p), "{p:?}");
            assert_eq!(size.index_of(p), None, "{p:?}");
        }
        assert_eq!(size.point_at(15), None);
        assert_eq!(size.point_at(usize::MAX), None);
    }

    #[test]
    fn a_side_of_i32_max_is_addressable() {
        let size = GridSize::new(i32::MAX.unsigned_abs(), 1);
        let last = Point::new(i32::MAX - 1, 0);
        assert!(size.contains(last));
        assert!(!size.contains(Point::new(i32::MAX, 0)));
        assert_eq!(
            size.index_of(last),
            Some(i32::MAX.unsigned_abs() as usize - 1)
        );
    }

    #[test]
    fn zero_sized_grids_are_empty() {
        for size in [
            GridSize::new(0, 3),
            GridSize::new(5, 0),
            GridSize::default(),
        ] {
            assert_eq!(size.area(), 0, "{size:?}");
            assert_eq!(size.tiles().count(), 0, "{size:?}");
            assert!(size.rows().all(|row| row.is_empty()), "{size:?}");
            assert_eq!(size.point_at(0), None, "{size:?}");
            assert!(!size.contains(Point::new(0, 0)), "{size:?}");
            for i in [0, 1, usize::MAX] {
                assert_eq!(size.north_of(i), None);
                assert_eq!(size.east_of(i), None);
                assert_eq!(size.south_of(i), None);
                assert_eq!(size.west_of(i), None);
            }
        }
    }

    #[test]
    #[should_panic(expected = "grid side exceeds i32::MAX")]
    fn new_rejects_a_side_past_i32_max_even_with_zero_area() {
        let _ = GridSize::new(i32::MAX.unsigned_abs() + 1, 0);
    }

    #[cfg(target_pointer_width = "32")]
    #[test]
    #[should_panic(expected = "grid side exceeds i32::MAX")]
    fn new_rejects_an_area_overflowing_usize() {
        let _ = GridSize::new(65536, 65536);
    }

    #[cfg(target_pointer_width = "32")]
    #[test]
    #[should_panic(expected = "grid side exceeds i32::MAX")]
    fn new_rejects_an_area_past_isize_max() {
        let _ = GridSize::new(65536, 32768);
    }

    #[test]
    fn try_from_checks_like_new() {
        assert_eq!(GridSize::try_from((5, 3)), Ok(GridSize::new(5, 3)));
        assert_eq!(
            GridSize::try_from((i32::MAX.unsigned_abs() + 1, 1)),
            Err(GridSizeError)
        );
        assert_ne!(GridSizeError.to_string(), "");
    }

    #[test]
    fn neighbours_match_point_arithmetic() {
        let size = GridSize::new(5, 3);
        for i in 0..size.area() {
            let p = size.point_at(i).unwrap();
            let expect = |d| size.index_of(p + d);
            assert_eq!(size.north_of(i), expect(DIR4[0]), "north of {i}");
            assert_eq!(size.east_of(i), expect(DIR4[1]), "east of {i}");
            assert_eq!(size.south_of(i), expect(DIR4[2]), "south of {i}");
            assert_eq!(size.west_of(i), expect(DIR4[3]), "west of {i}");
        }
        for i in [
            size.area(),
            size.area() + 1,
            size.area() + size.stride(),
            usize::MAX,
        ] {
            assert_eq!(size.north_of(i), None, "{i}");
            assert_eq!(size.east_of(i), None, "{i}");
            assert_eq!(size.south_of(i), None, "{i}");
            assert_eq!(size.west_of(i), None, "{i}");
        }
    }

    #[test]
    fn rows_cover_every_index_in_order() {
        let size = GridSize::new(5, 3);
        let rows: Vec<_> = size.rows().collect();
        assert_eq!(rows, vec![0..5, 5..10, 10..15]);
        assert!(size.rows().flatten().eq(0..size.area()));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_takes_a_map_or_a_sequence_and_checks_the_sides() {
        use serde::de::value::{Error, MapDeserializer, SeqDeserializer};
        use serde::Deserialize;

        let map = MapDeserializer::<_, Error>::new(vec![("w", 31u32), ("h", 16)].into_iter());
        assert_eq!(GridSize::deserialize(map), Ok(GridSize::new(31, 16)));
        let seq = SeqDeserializer::<_, Error>::new(vec![31u32, 16].into_iter());
        assert_eq!(GridSize::deserialize(seq), Ok(GridSize::new(31, 16)));

        let map =
            MapDeserializer::<_, Error>::new(vec![("w", 2_147_483_648u32), ("h", 1)].into_iter());
        assert!(GridSize::deserialize(map).is_err());
        let seq = SeqDeserializer::<_, Error>::new(vec![2_147_483_648u32, 1].into_iter());
        assert!(GridSize::deserialize(seq).is_err());
    }
}
