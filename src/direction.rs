use std::cmp::Ordering::{Equal, Greater, Less};
use std::ops::Mul;

use num_traits::{Float, FloatConst};

use super::{Point, Vec2};

/// The eight directions clockwise from North; index × π/4 is the angle (see [`Direction`]).
pub const DIR8: [Direction; 8] = [
    Direction::North,
    Direction::NorthEast,
    Direction::East,
    Direction::SouthEast,
    Direction::South,
    Direction::SouthWest,
    Direction::West,
    Direction::NorthWest,
];

/// `Here` followed by [`DIR8`].
pub const DIR9: [Direction; 9] = [
    Direction::Here,
    Direction::North,
    Direction::NorthEast,
    Direction::East,
    Direction::SouthEast,
    Direction::South,
    Direction::SouthWest,
    Direction::West,
    Direction::NorthWest,
];

/// The four orthogonal directions — every other one of [`DIR8`], clockwise from North. They are
/// the neighbours that share an edge with a tile rather than only a corner.
pub const DIR4: [Direction; 4] = [
    Direction::North,
    Direction::East,
    Direction::South,
    Direction::West,
];

/// Each orthogonal direction paired with the one that faces back at it, in
/// [`DIR4`] order.
///
/// The pair is what a symmetric edge needs: stepping `d` from a tile and then
/// asking the neighbour about `opposite` is the same edge read from both ends.
/// Walking this instead of [`DIR4`] plus a call to [`Direction::opposite`] keeps
/// the two halves of an edge visibly together at the call site.
pub const DIR4_OPPOSITE: [(Direction, Direction); 4] = [
    (Direction::North, Direction::South),
    (Direction::East, Direction::West),
    (Direction::South, Direction::North),
    (Direction::West, Direction::East),
];

/// The half of [`DIR4_OPPOSITE`] that visits every undirected orthogonal edge of
/// a grid **exactly once** when swept over all tiles.
///
/// East and South suffice: a tile's West edge is its west neighbour's East edge,
/// and its North edge is its north neighbour's South edge, so both are already
/// covered by the time the sweep reaches them. Use this — not [`DIR4`] — for a
/// pass that must not process an edge twice, such as one moving a conserved
/// quantity across it.
pub const DIR2_EDGE_ONCE: [(Direction, Direction); 2] = [
    (Direction::East, Direction::West),
    (Direction::South, Direction::North),
];

/// One of the eight compass directions, or `Here`.
///
/// Screen convention: y grows downward, so North is `(0, -1)`. Turns go clockwise from North,
/// in [`DIR8`] order.
///
/// Angles follow the same convention: North is 0 and each [`DIR8`] step adds π/4, clockwise.
/// [`Direction::from_angle`] and [`Direction::nearest`] round to the nearest direction; when
/// `θ / (π/4)` is exactly halfway between two integers the clockwise one wins.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Direction {
    Here,
    North,
    NorthEast,
    #[default]
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

impl Direction {
    #[must_use]
    pub fn is_default(self) -> bool {
        self == Self::default()
    }

    #[must_use]
    pub fn all() -> [Direction; 8] {
        DIR8
    }

    #[must_use]
    pub fn all_with_here() -> [Direction; 9] {
        DIR9
    }

    /// The four edge-sharing neighbours, clockwise from North — see [`DIR4`].
    #[must_use]
    pub fn orthogonals() -> [Direction; 4] {
        DIR4
    }

    /// The direction facing back along this one. `Here` has no opposite and is
    /// returned unchanged, which is what makes this total.
    #[inline]
    #[must_use]
    pub const fn opposite(self) -> Self {
        match self {
            Direction::Here => Direction::Here,
            Direction::North => Direction::South,
            Direction::NorthEast => Direction::SouthWest,
            Direction::East => Direction::West,
            Direction::SouthEast => Direction::NorthWest,
            Direction::South => Direction::North,
            Direction::SouthWest => Direction::NorthEast,
            Direction::West => Direction::East,
            Direction::NorthWest => Direction::SouthEast,
        }
    }

    /// Position in [`DIR8`]; `None` for `Here`.
    const fn dir8_index(self) -> Option<usize> {
        match self {
            Direction::Here => None,
            Direction::North => Some(0),
            Direction::NorthEast => Some(1),
            Direction::East => Some(2),
            Direction::SouthEast => Some(3),
            Direction::South => Some(4),
            Direction::SouthWest => Some(5),
            Direction::West => Some(6),
            Direction::NorthWest => Some(7),
        }
    }

    /// Turned by `eighths` of a circle, clockwise when positive. `Here` stays `Here`.
    #[inline]
    #[must_use]
    pub const fn rotated(self, eighths: i32) -> Self {
        match self.dir8_index() {
            None => Direction::Here,
            Some(i) => DIR8[(i + eighths.rem_euclid(8) as usize) % 8],
        }
    }

    #[inline]
    #[must_use]
    pub const fn rotate_cw(self) -> Self {
        self.rotated(1)
    }

    #[inline]
    #[must_use]
    pub const fn rotate_ccw(self) -> Self {
        self.rotated(-1)
    }

    /// This direction's angle in `[0, 2π)` (see [`Direction`]); `None` for `Here`.
    #[must_use]
    pub fn angle<T: Float + FloatConst>(self) -> Option<T> {
        let i = self.dir8_index()?;
        Some(T::from(i).unwrap_or_else(T::zero) * T::FRAC_PI_4())
    }

    /// The direction nearest angle `theta` (see [`Direction`]); `Here` for NaN or ±∞. Very large
    /// angles are only as precise as their reduction by 2π.
    #[must_use]
    pub fn from_angle<T: Float + FloatConst>(theta: T) -> Self {
        if !theta.is_finite() {
            return Direction::Here;
        }
        let one = T::one();
        let half = one / (one + one);
        let eight = T::from(8).unwrap_or(one);
        let x = (theta % T::TAU()) / T::FRAC_PI_4();
        let f = x.floor();
        let k = if x - f >= half { f + one } else { f };
        let m = k - eight * (k / eight).floor();
        DIR8[m.to_usize().unwrap_or(0) % 8]
    }

    /// The direction nearest vector `v` (see [`Direction`]); `Here` for a zero or NaN vector.
    #[must_use]
    pub fn nearest<T: Float + FloatConst>(v: vek::Vec2<T>) -> Self {
        if v.x.is_nan() || v.y.is_nan() || (v.x.is_zero() && v.y.is_zero()) {
            return Direction::Here;
        }
        Self::from_angle(v.x.atan2(-v.y))
    }

    /// True for the four edge-sharing directions. `Here` is neither orthogonal
    /// nor diagonal.
    #[inline]
    #[must_use]
    pub fn is_orthogonal(self) -> bool {
        matches!(
            self,
            Direction::North | Direction::East | Direction::South | Direction::West
        )
    }

    #[cfg(feature = "rand")]
    pub fn random<R: rand::Rng + rand::RngExt + ?Sized>(rng: &mut R, include_here: bool) -> Self {
        if include_here {
            DIR9[rng.random_range(0..DIR9.len())]
        } else {
            DIR8[rng.random_range(0..DIR8.len())]
        }
    }

    #[inline]
    #[must_use]
    pub fn from_delta(dx: i32, dy: i32) -> Self {
        match (dx.cmp(&0), dy.cmp(&0)) {
            (Less, Less) => Direction::NorthWest,
            (Less, Equal) => Direction::West,
            (Less, Greater) => Direction::SouthWest,
            (Equal, Less) => Direction::North,
            (Equal, Equal) => Direction::Here,
            (Equal, Greater) => Direction::South,
            (Greater, Less) => Direction::NorthEast,
            (Greater, Equal) => Direction::East,
            (Greater, Greater) => Direction::SouthEast,
        }
    }

    #[inline]
    #[must_use]
    pub fn dx(self) -> i32 {
        match self {
            Direction::NorthWest | Direction::West | Direction::SouthWest => -1,
            Direction::NorthEast | Direction::East | Direction::SouthEast => 1,
            Direction::North | Direction::South | Direction::Here => 0,
        }
    }

    #[inline]
    #[must_use]
    pub fn dy(self) -> i32 {
        match self {
            Direction::NorthEast | Direction::North | Direction::NorthWest => -1,
            Direction::SouthEast | Direction::South | Direction::SouthWest => 1,
            Direction::East | Direction::West | Direction::Here => 0,
        }
    }

    #[inline]
    #[must_use]
    pub fn is_here(self) -> bool {
        self == Direction::Here
    }

    #[inline]
    #[must_use]
    pub fn is_diagonal(self) -> bool {
        matches!(
            self,
            Direction::NorthEast
                | Direction::SouthEast
                | Direction::SouthWest
                | Direction::NorthWest
        )
    }
}

impl From<(i32, i32)> for Direction {
    fn from((dx, dy): (i32, i32)) -> Self {
        Self::from_delta(dx, dy)
    }
}

impl From<Point> for Direction {
    fn from(point: Point) -> Self {
        Self::from_delta(point.x, point.y)
    }
}

impl From<Direction> for Vec2 {
    #[allow(clippy::cast_precision_loss)]
    fn from(dir: Direction) -> Self {
        Vec2::new(dir.dx() as f32, dir.dy() as f32)
    }
}

impl Mul<i32> for Direction {
    type Output = Point;

    /// `n` steps along this direction, as an offset.
    #[inline]
    fn mul(self, n: i32) -> Point {
        Point::new(self.dx() * n, self.dy() * n)
    }
}

#[cfg(test)]
mod tests {
    use super::{Direction, Point, DIR2_EDGE_ONCE, DIR4, DIR4_OPPOSITE, DIR8, DIR9};

    #[test]
    fn from_delta() {
        let dir = Direction::from_delta(10, 20);
        assert!(matches!(dir, Direction::SouthEast));
    }

    #[test]
    fn from_tuple() {
        let dir = Direction::from((10, 20));
        assert!(matches!(dir, Direction::SouthEast));
    }

    #[test]
    fn from_point() {
        let dir = Direction::from(Point::new(10, 20));
        assert!(matches!(dir, Direction::SouthEast));
    }

    #[test]
    fn from_point_diff() {
        let pt = Point::new(1, 2);
        let dir = pt.direction_to(Point::new(3, 4));
        assert!(matches!(dir, Direction::SouthEast));
    }

    #[test]
    fn dir8_runs_clockwise_from_north() {
        assert_eq!(
            DIR8,
            [
                Direction::North,
                Direction::NorthEast,
                Direction::East,
                Direction::SouthEast,
                Direction::South,
                Direction::SouthWest,
                Direction::West,
                Direction::NorthWest,
            ]
        );
        assert_eq!(DIR9[0], Direction::Here);
        assert_eq!(DIR9[1..], DIR8);
    }

    #[test]
    fn dir4_is_every_other_dir8() {
        assert_eq!(DIR4, [DIR8[0], DIR8[2], DIR8[4], DIR8[6]]);
        assert_eq!(DIR4, Direction::orthogonals());
    }

    #[test]
    fn rotate_cw_is_an_eighth_turn_clockwise() {
        // Two eighths make a clockwise quarter: (x, y) → (−y, x) with y pointing down.
        for d in DIR8 {
            let q = d.rotate_cw().rotate_cw();
            assert_eq!((q.dx(), q.dy()), (-d.dy(), d.dx()), "{d:?}");
            assert_eq!(d.rotate_cw().rotate_ccw(), d, "{d:?}");
        }
        assert_eq!(Direction::North.rotate_cw(), Direction::NorthEast);
        assert_eq!(Direction::North.rotated(2), Direction::East);
    }

    #[test]
    fn rotated_is_repeated_eighth_turns() {
        for d in DIR8 {
            for k in -16..16_i32 {
                let mut e = d;
                for _ in 0..k.unsigned_abs() {
                    e = if k > 0 { e.rotate_cw() } else { e.rotate_ccw() };
                }
                assert_eq!(d.rotated(k), e, "{d:?} by {k}");
            }
            assert_eq!(d.rotated(8), d);
            assert_eq!(d.rotated(i32::MAX), d.rotate_ccw(), "i32::MAX ≡ 7 (mod 8)");
            assert_eq!(d.rotated(i32::MIN), d, "i32::MIN ≡ 0 (mod 8)");
        }
        assert_eq!(Direction::Here.rotated(3), Direction::Here);
        assert_eq!(Direction::Here.rotate_cw(), Direction::Here);
    }

    const _: Direction = Direction::North.rotated(i32::MIN);

    #[cfg(feature = "rand")]
    #[test]
    fn random_draws_from_dir8_or_dir9() {
        use rand::SeedableRng;
        let mut rng = rand::rngs::Xoshiro256PlusPlus::seed_from_u64(7);
        let without: Vec<Direction> = (0..400)
            .map(|_| Direction::random(&mut rng, false))
            .collect();
        assert!(!without.contains(&Direction::Here));
        assert!(DIR8.iter().all(|d| without.contains(d)));
        let with: Vec<Direction> = (0..400)
            .map(|_| Direction::random(&mut rng, true))
            .collect();
        assert!(with.contains(&Direction::Here));
    }

    #[test]
    fn dir4_turns_clockwise() {
        // Each step is a quarter turn the same way round, which is what lets a
        // caller index this array to rotate rather than to enumerate.
        // Screen axes: y grows downward, so a clockwise quarter turn maps
        // (x, y) → (−y, x). North (0, −1) → East (1, 0) is the first step.
        for i in 0..DIR4.len() {
            let (a, b) = (DIR4[i], DIR4[(i + 1) % DIR4.len()]);
            assert_eq!(
                (b.dx(), b.dy()),
                (-a.dy(), a.dx()),
                "{a:?} → {b:?} is not a clockwise quarter turn"
            );
        }
    }

    #[test]
    fn opposite_is_an_involution_and_negates_the_delta() {
        for d in DIR9 {
            assert_eq!(
                d.opposite().opposite(),
                d,
                "{d:?} is not its own round trip"
            );
            assert_eq!(
                (d.opposite().dx(), d.opposite().dy()),
                (-d.dx(), -d.dy()),
                "{d:?}'s opposite does not reverse its delta"
            );
        }
        assert_eq!(Direction::Here.opposite(), Direction::Here);
    }

    #[test]
    fn dir4_opposite_pairs_each_direction_with_its_own_opposite() {
        assert_eq!(DIR4_OPPOSITE.len(), DIR4.len());
        for (i, (d, opp)) in DIR4_OPPOSITE.into_iter().enumerate() {
            assert_eq!(d, DIR4[i], "the pair list must follow DIR4's order");
            assert_eq!(opp, d.opposite());
        }
    }

    #[test]
    fn edge_once_covers_every_undirected_edge_exactly_once() {
        // Sweep a 3×3 grid: every unordered tile pair that shares an edge must be
        // produced once and only once.
        let mut seen: Vec<(Point, Point)> = Vec::new();
        for y in 0..3 {
            for x in 0..3 {
                let p = Point::new(x, y);
                for (d, opp) in DIR2_EDGE_ONCE {
                    assert_eq!(opp, d.opposite(), "the pair must face back at itself");
                    let n = p + d;
                    if (0..3).contains(&n.x) && (0..3).contains(&n.y) {
                        seen.push((p, n));
                    }
                }
            }
        }
        // 3×3 has 12 shared edges: 2 per row × 3 rows, plus 2 per column × 3.
        assert_eq!(seen.len(), 12, "each edge produced once: {seen:?}");
        let mut normalized: Vec<(Point, Point)> = seen
            .iter()
            .map(|&(a, b)| {
                if (a.x, a.y) <= (b.x, b.y) {
                    (a, b)
                } else {
                    (b, a)
                }
            })
            .collect();
        normalized.sort_by_key(|&(a, b)| (a.x, a.y, b.x, b.y));
        let before = normalized.len();
        normalized.dedup();
        assert_eq!(normalized.len(), before, "an edge was visited twice");
    }

    #[test]
    fn is_orthogonal_and_is_diagonal_partition_the_eight() {
        for d in DIR8 {
            assert!(
                d.is_orthogonal() != d.is_diagonal(),
                "{d:?} must be exactly one of the two"
            );
        }
        assert!(!Direction::Here.is_orthogonal());
        assert!(!Direction::Here.is_diagonal());
    }

    fn angles_hold<T>(below_tie: T, big: T)
    where
        T: num_traits::Float + num_traits::FloatConst + std::fmt::Debug,
    {
        let two = T::one() + T::one();
        for (i, d) in (0u8..).zip(DIR8) {
            let expected = T::from(i).unwrap() * T::FRAC_PI_4();
            assert_eq!(d.angle::<T>(), Some(expected), "{d:?}");
            assert_eq!(Direction::from_angle(expected), d, "{d:?} round trip");
        }
        assert_eq!(Direction::Here.angle::<T>(), None);
        assert_eq!(Direction::North.angle::<T>(), Some(T::zero()));
        assert_eq!(Direction::East.angle::<T>(), Some(T::FRAC_PI_2()));
        assert_eq!(Direction::South.angle::<T>(), Some(T::PI()));

        assert_eq!(Direction::from_angle(T::FRAC_PI_8()), Direction::NorthEast);
        assert_eq!(Direction::from_angle(-T::FRAC_PI_8()), Direction::North);
        assert_eq!(Direction::from_angle(below_tie), Direction::North);
        for theta in [T::max_value(), T::max_value() / two, big] {
            assert_ne!(Direction::from_angle(theta), Direction::Here, "{theta:?}");
            assert_ne!(Direction::from_angle(-theta), Direction::Here, "{theta:?}");
        }
        for theta in [T::nan(), T::infinity(), T::neg_infinity()] {
            assert_eq!(Direction::from_angle(theta), Direction::Here, "{theta:?}");
        }

        for d in DIR8 {
            let v = vek::Vec2::new(T::from(d.dx()).unwrap(), T::from(d.dy()).unwrap());
            assert_eq!(Direction::nearest(v), d, "{d:?}");
        }
        let (z, nz) = (T::zero(), -T::zero());
        for (x, y) in [(z, z), (nz, z), (z, nz), (nz, nz)] {
            assert_eq!(Direction::nearest(vek::Vec2::new(x, y)), Direction::Here);
        }
        assert_eq!(
            Direction::nearest(vek::Vec2::new(T::nan(), T::one())),
            Direction::Here
        );
        assert_eq!(
            Direction::nearest(vek::Vec2::new(T::one(), T::nan())),
            Direction::Here
        );
        assert_eq!(
            Direction::nearest(vek::Vec2::new(T::infinity(), T::infinity())),
            Direction::SouthEast
        );
    }

    #[test]
    fn angles_f32() {
        let below_tie = f32::from_bits(std::f32::consts::FRAC_PI_8.to_bits() - 1);
        angles_hold(below_tie, 1e30_f32);
    }

    #[test]
    fn angles_f64() {
        let below_tie = f64::from_bits(std::f64::consts::FRAC_PI_8.to_bits() - 1);
        angles_hold(below_tie, 1e300_f64);
    }

    #[test]
    fn direction_times_n_is_n_steps() {
        for d in DIR9 {
            assert_eq!(d * 3, Point::new(d.dx() * 3, d.dy() * 3), "{d:?}");
        }
        assert_eq!(Direction::Here * 5, Point::new(0, 0));
        assert_eq!(Direction::East * -2, Point::new(-2, 0));
        assert_eq!(Point::new(1, 1) + Direction::South * 4, Point::new(1, 5));
    }
}
