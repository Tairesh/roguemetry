use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use vek::num_traits::Zero;

use super::{Direction, Vec2};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Helper struct defining a 2D point in space.
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    #[must_use]
    /// Create a new point from i32
    pub const fn new(x: i32, y: i32) -> Self {
        Point { x, y }
    }

    #[inline]
    #[must_use]
    /// Try to create a new point from `TryInto<i32>`
    pub fn try_new<T>(x: T, y: T) -> Point
    where
        T: TryInto<i32>,
    {
        Point {
            x: x.try_into().ok().unwrap_or(0),
            y: y.try_into().ok().unwrap_or(0),
        }
    }

    #[cfg(feature = "rand")]
    #[inline]
    #[must_use]
    /// Create a random point within a range
    pub fn random<R: rand::Rng + rand::RngExt + ?Sized>(
        rng: &mut R,
        horizontal: std::ops::Range<i32>,
        vertical: std::ops::Range<i32>,
    ) -> Self {
        let x = rng.random_range(horizontal);
        let y = rng.random_range(vertical);

        Self::new(x, y)
    }

    #[inline]
    #[must_use]
    #[allow(clippy::cast_sign_loss)]
    /// Helper for map index conversion
    pub fn to_index(self, width: i32) -> Option<usize> {
        if self.x < 0 || self.y < 0 || self.x >= width {
            None
        } else {
            Some(((self.y * width) + self.x) as usize)
        }
    }

    #[inline]
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_possible_wrap)]
    /// Helper for map index conversion
    pub fn from_index(index: usize, width: i32) -> Point {
        Point::new(index as i32 % width, index as i32 / width)
    }

    #[must_use]
    /// Direction to other point
    pub fn direction_to(self, other: Point) -> Direction {
        Direction::from(other - self)
    }

    #[inline]
    #[must_use]
    /// Square distance to other point
    pub fn square_distance_to(self, other: Self) -> u32 {
        let dx = self.x.abs_diff(other.x);
        let dy = self.y.abs_diff(other.y);
        dx * dx + dy * dy
    }

    #[inline]
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    /// Distance (pythagorean) to other point
    pub fn distance_to(self, other: Self) -> f32 {
        (self.square_distance_to(other) as f32).sqrt()
    }

    #[must_use]
    /// Points between self and other
    pub fn line_to(self, other: Point) -> Vec<Point> {
        line_drawing::Bresenham::new(self.into(), other.into())
            .map(Self::from)
            .collect()
    }
}

impl Default for Point {
    /// Create a zero point
    fn default() -> Self {
        Self::zero()
    }
}

impl Zero for Point {
    /// Create a zero point
    #[inline]
    fn zero() -> Self {
        Self::new(0, 0)
    }

    /// Check if point is zero
    fn is_zero(&self) -> bool {
        self == &Self::zero()
    }
}

impl From<Point> for (i32, i32) {
    fn from(pos: Point) -> Self {
        (pos.x, pos.y)
    }
}

impl From<(i32, i32)> for Point {
    fn from((x, y): (i32, i32)) -> Self {
        Self::new(x, y)
    }
}

impl From<Point> for Vec2 {
    #[allow(clippy::cast_precision_loss)]
    fn from(point: Point) -> Self {
        Self::new(point.x as f32, point.y as f32)
    }
}

impl From<Vec2> for Point {
    #[allow(clippy::cast_possible_truncation)]
    fn from(vec: Vec2) -> Self {
        Self::new(vec.x.round() as i32, vec.y.round() as i32)
    }
}

impl From<Direction> for Point {
    fn from(dir: Direction) -> Self {
        Self::new(dir.dx(), dir.dy())
    }
}

impl Add<Direction> for Point {
    type Output = Point;

    fn add(self, rhs: Direction) -> Self::Output {
        Self::new(self.x + rhs.dx(), self.y + rhs.dy())
    }
}

impl AddAssign<Direction> for Point {
    fn add_assign(&mut self, rhs: Direction) {
        self.x += rhs.dx();
        self.y += rhs.dy();
    }
}

impl Add<(i32, i32)> for Point {
    type Output = Point;

    fn add(self, (dx, dy): (i32, i32)) -> Self::Output {
        Self::new(self.x + dx, self.y + dy)
    }
}

impl AddAssign<(i32, i32)> for Point {
    fn add_assign(&mut self, (dx, dy): (i32, i32)) {
        self.x += dx;
        self.y += dy;
    }
}

impl Add<Point> for Point {
    type Output = Point;

    fn add(self, rhs: Point) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl AddAssign<Point> for Point {
    fn add_assign(&mut self, rhs: Point) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Add<Vec2> for Point {
    type Output = Point;

    fn add(self, rhs: Vec2) -> Self::Output {
        (Vec2::from(self) + rhs).into()
    }
}

impl AddAssign<Vec2> for Point {
    fn add_assign(&mut self, rhs: Vec2) {
        *self = *self + rhs;
    }
}

impl Sub<Direction> for Point {
    type Output = Point;

    fn sub(self, rhs: Direction) -> Self::Output {
        Self::new(self.x - rhs.dx(), self.y - rhs.dy())
    }
}

impl SubAssign<Direction> for Point {
    fn sub_assign(&mut self, rhs: Direction) {
        self.x -= rhs.dx();
        self.y -= rhs.dy();
    }
}

impl Sub<(i32, i32)> for Point {
    type Output = Point;

    fn sub(self, (dx, dy): (i32, i32)) -> Self::Output {
        Self::new(self.x - dx, self.y - dy)
    }
}

impl SubAssign<(i32, i32)> for Point {
    fn sub_assign(&mut self, (dx, dy): (i32, i32)) {
        self.x -= dx;
        self.y -= dy;
    }
}

impl Sub<Point> for Point {
    type Output = Point;

    fn sub(self, rhs: Point) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl SubAssign<Point> for Point {
    fn sub_assign(&mut self, rhs: Point) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Sub<Vec2> for Point {
    type Output = Point;

    fn sub(self, rhs: Vec2) -> Self::Output {
        (Vec2::from(self) - rhs).into()
    }
}

impl SubAssign<Vec2> for Point {
    fn sub_assign(&mut self, rhs: Vec2) {
        *self = *self - rhs;
    }
}

impl Mul<i32> for Point {
    type Output = Point;

    fn mul(self, rhs: i32) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl MulAssign<i32> for Point {
    fn mul_assign(&mut self, rhs: i32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl Mul<(i32, i32)> for Point {
    type Output = Point;

    fn mul(self, (mx, my): (i32, i32)) -> Self::Output {
        Self::new(self.x * mx, self.y * my)
    }
}

impl MulAssign<(i32, i32)> for Point {
    fn mul_assign(&mut self, (mx, my): (i32, i32)) {
        self.x *= mx;
        self.y *= my;
    }
}

impl Mul<Point> for Point {
    type Output = Point;

    fn mul(self, rhs: Point) -> Self::Output {
        Self::new(self.x * rhs.x, self.y * rhs.y)
    }
}

impl MulAssign<Point> for Point {
    fn mul_assign(&mut self, rhs: Point) {
        self.x *= rhs.x;
        self.y *= rhs.y;
    }
}

impl Mul<f32> for Point {
    type Output = Point;

    fn mul(self, rhs: f32) -> Self::Output {
        (Vec2::from(self) * rhs).into()
    }
}

impl MulAssign<f32> for Point {
    fn mul_assign(&mut self, rhs: f32) {
        *self = *self * rhs;
    }
}

impl Mul<(f32, f32)> for Point {
    type Output = Point;

    fn mul(self, (mx, my): (f32, f32)) -> Self::Output {
        (Vec2::from(self) * Vec2::new(mx, my)).into()
    }
}

impl MulAssign<(f32, f32)> for Point {
    fn mul_assign(&mut self, (mx, my): (f32, f32)) {
        *self = *self * Vec2::new(mx, my);
    }
}

impl Mul<Vec2> for Point {
    type Output = Point;

    fn mul(self, rhs: Vec2) -> Self::Output {
        (rhs * Vec2::from(self)).into()
    }
}

impl MulAssign<Vec2> for Point {
    fn mul_assign(&mut self, rhs: Vec2) {
        *self = *self * rhs;
    }
}

impl Div<i32> for Point {
    type Output = Point;

    fn div(self, rhs: i32) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs)
    }
}

impl DivAssign<i32> for Point {
    fn div_assign(&mut self, rhs: i32) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

impl Div<(i32, i32)> for Point {
    type Output = Point;

    fn div(self, (mx, my): (i32, i32)) -> Self::Output {
        Self::new(self.x / mx, self.y / my)
    }
}

impl DivAssign<(i32, i32)> for Point {
    fn div_assign(&mut self, (mx, my): (i32, i32)) {
        self.x /= mx;
        self.y /= my;
    }
}

impl Div<Point> for Point {
    type Output = Point;

    fn div(self, rhs: Point) -> Self::Output {
        Self::new(self.x / rhs.x, self.y / rhs.y)
    }
}

impl DivAssign<Point> for Point {
    fn div_assign(&mut self, rhs: Point) {
        self.x /= rhs.x;
        self.y /= rhs.y;
    }
}

impl Div<f32> for Point {
    type Output = Point;

    fn div(self, rhs: f32) -> Self::Output {
        (Vec2::from(self) / rhs).into()
    }
}

impl DivAssign<f32> for Point {
    fn div_assign(&mut self, rhs: f32) {
        *self = *self / rhs;
    }
}

impl Div<(f32, f32)> for Point {
    type Output = Point;

    fn div(self, (mx, my): (f32, f32)) -> Self::Output {
        (Vec2::from(self) / Vec2::new(mx, my)).into()
    }
}

impl DivAssign<(f32, f32)> for Point {
    fn div_assign(&mut self, (mx, my): (f32, f32)) {
        *self = *self / Vec2::new(mx, my);
    }
}

impl Div<Vec2> for Point {
    type Output = Point;

    fn div(self, rhs: Vec2) -> Self::Output {
        (Vec2::from(self) / rhs).into()
    }
}

impl DivAssign<Vec2> for Point {
    fn div_assign(&mut self, rhs: Vec2) {
        *self = *self / rhs;
    }
}

impl Neg for Point {
    type Output = Point;

    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y)
    }
}

impl PartialEq<(i32, i32)> for Point {
    fn eq(&self, other: &(i32, i32)) -> bool {
        self.x == other.0 && self.y == other.1
    }
}

#[cfg(test)]
mod tests {
    use super::{Direction, Point, Vec2};

    #[test]
    fn index_converting() {
        let pt = Point::new(1, 2);
        assert_eq!(21, pt.to_index(10).unwrap());
        let pt2 = Point::from_index(21, 10);
        assert_eq!(pt, pt2);
        let pt = Point::new(-1, 2);
        assert!(pt.to_index(10).is_none());
        let pt = Point::new(1, -2);
        assert!(pt.to_index(10).is_none());
        let pt = Point::new(10, 2);
        assert!(pt.to_index(10).is_none());
    }

    #[test]
    fn add_direction_to_point() {
        let mut pt = Point::new(1, 2);
        pt += Direction::NorthWest;
        assert_eq!(0, pt.x);
        assert_eq!(1, pt.y);
    }

    #[test]
    fn mul_point_to_float_tuple() {
        let pt = Point::new(1, 2);
        let pt2 = pt * (1.4, 2.2);
        assert_eq!(1, pt2.x);
        assert_eq!(4, pt2.y);
    }

    #[test]
    fn mul_point_to_vec() {
        let pt = Point::new(2, 3);
        let pt2 = pt * Vec2::new(1.5, 2.0);
        assert_eq!(3, pt2.x);
        assert_eq!(6, pt2.y);
    }

    #[test]
    fn test_dist() {
        let pt = Point::new(1, 2);
        let pt2 = Point::new(4, 6);
        assert_eq!(25, pt.square_distance_to(pt2));
        assert!(f32::abs(pt.distance_to(pt2) - 5.0) < f32::EPSILON);
    }

    #[test]
    fn mul_assign() {
        let mut pt = Point::new(1, 2);
        pt *= (1.5, 2.0);
        assert_eq!(2, pt.x);
        assert_eq!(4, pt.y);

        let mut pt = Point::new(1, 2);
        pt *= Vec2::new(1.5, 2.0);
        assert_eq!(2, pt.x);
        assert_eq!(4, pt.y);
    }

    #[test]
    fn div_assign() {
        let mut pt = Point::new(1, 2);
        pt /= (3.0, 2.0);
        assert_eq!(0, pt.x);
        assert_eq!(1, pt.y);

        let mut pt = Point::new(1, 2);
        pt /= Vec2::new(3.0, 2.0);
        assert_eq!(0, pt.x);
        assert_eq!(1, pt.y);
    }

    #[test]
    fn div_assign_point() {
        let mut pt = Point::new(1, 2);
        pt /= Point::new(1, 2);
        assert_eq!(1, pt.x);
        assert_eq!(1, pt.y);
    }

    #[test]
    fn div_assign_f32() {
        let mut pt = Point::new(1, 2);
        pt /= 3.0;
        assert_eq!(0, pt.x);
        assert_eq!(1, pt.y);
    }

    #[test]
    fn div_point() {
        let pt = Point::new(1, 2);
        let pt2 = pt / Point::new(1, 2);
        assert_eq!(1, pt2.x);
        assert_eq!(1, pt2.y);
    }

    #[test]
    fn test_line() {
        let pt = Point::new(0, 0);
        let pt2 = Point::new(5, 5);
        assert_eq!(
            pt.line_to(pt2),
            [(0, 0), (1, 1), (2, 2), (3, 3), (4, 4), (5, 5)]
        )
    }
}
