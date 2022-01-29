#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub const fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub const fn from_tuple(point_tuple: (f64, f64)) -> Self {
        Self {
            x: point_tuple.0,
            y: point_tuple.1,
        }
    }
}
