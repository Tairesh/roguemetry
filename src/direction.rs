use std::cmp::Ordering::{Equal, Greater, Less};

use super::{Point, Vec2};

pub const DIR8: [Direction; 8] = [
    Direction::East,
    Direction::SouthEast,
    Direction::South,
    Direction::SouthWest,
    Direction::West,
    Direction::NorthWest,
    Direction::North,
    Direction::NorthEast,
];

pub const DIR9: [Direction; 9] = [
    Direction::Here,
    Direction::East,
    Direction::SouthEast,
    Direction::South,
    Direction::SouthWest,
    Direction::West,
    Direction::NorthWest,
    Direction::North,
    Direction::NorthEast,
];

/// The four orthogonal directions, clockwise from North — the neighbours that
/// share an edge with a tile rather than only a corner.
///
/// Clockwise like [`DIR8`], but entered at North rather than East, because that
/// is the order the four are conventionally named in. It is therefore a rotation
/// of `DIR8`'s orthogonal subsequence, not a slice of it.
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

    /// True for the four edge-sharing directions. `Here` is neither orthogonal
    /// nor diagonal.
    #[must_use]
    pub fn is_orthogonal(self) -> bool {
        matches!(
            self,
            Direction::North | Direction::East | Direction::South | Direction::West
        )
    }

    #[cfg(feature = "rand")]
    pub fn random<R: rand::Rng + rand::RngExt + ?Sized>(rng: &mut R, include_here: bool) -> Self {
        match rng.random_range(0..=if include_here { 8 } else { 7 }) {
            0 => Direction::East,
            1 => Direction::SouthEast,
            2 => Direction::South,
            3 => Direction::SouthWest,
            4 => Direction::West,
            5 => Direction::NorthWest,
            6 => Direction::North,
            7 => Direction::NorthEast,
            8 => Direction::Here,
            _ => unreachable!(),
        }
    }

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

    #[must_use]
    pub fn dx(self) -> i32 {
        match self {
            Direction::NorthWest | Direction::West | Direction::SouthWest => -1,
            Direction::NorthEast | Direction::East | Direction::SouthEast => 1,
            Direction::North | Direction::South | Direction::Here => 0,
        }
    }

    #[must_use]
    pub fn dy(self) -> i32 {
        match self {
            Direction::NorthEast | Direction::North | Direction::NorthWest => -1,
            Direction::SouthEast | Direction::South | Direction::SouthWest => 1,
            Direction::East | Direction::West | Direction::Here => 0,
        }
    }

    #[must_use]
    pub fn is_here(self) -> bool {
        self == Direction::Here
    }

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
    fn dir4_is_the_orthogonal_half_of_dir8() {
        let mut from_four = DIR4;
        let mut from_eight: Vec<Direction> =
            DIR8.into_iter().filter(|d| d.is_orthogonal()).collect();
        from_four.sort_by_key(|d| format!("{d:?}"));
        from_eight.sort_by_key(|d| format!("{d:?}"));
        assert_eq!(
            from_four.to_vec(),
            from_eight,
            "same four, whatever the order"
        );
        assert_eq!(DIR4, Direction::orthogonals());
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
}
