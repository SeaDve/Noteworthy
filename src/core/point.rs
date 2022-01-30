/// Describes a point with two coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    /// A point fixed at origin (0, 0).
    pub const ZERO: Self = Self::new(0.0, 0.0);

    /// Construct a point with `x` and `y` coordinates.
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Construct a point from tuple (x, y).
    pub const fn from_tuple(point_tuple: (f64, f64)) -> Self {
        Self {
            x: point_tuple.0,
            y: point_tuple.1,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn zero() {
        let point = Point::ZERO;
        assert_eq!(point.x, 0.0);
        assert_eq!(point.y, 0.0);
    }

    #[test]
    fn new() {
        let point = Point::new(1.0, 5.0);
        assert_eq!(point.x, 1.0);
        assert_eq!(point.y, 5.0);
    }

    #[test]
    fn tuple() {
        let point_tuple = (1.0, 5.0);
        let point = Point::from_tuple(point_tuple);
        assert_eq!(point.x, 1.0);
        assert_eq!(point.y, 5.0);
    }
}
