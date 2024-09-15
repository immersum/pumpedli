use embedded_graphics::prelude::*;

#[derive(Clone, Copy)]
pub enum Position {
    Top,
    TopLeft,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomRight,
    Bottom,
}

impl From<Position> for Point {
    fn from(value: Position) -> Self {
        Point::from(match value {
            Position::Top => (0, -90),
            Position::TopLeft => (-39, -45),
            Position::TopRight => (39, -45),
            Position::CenterLeft => (-78, 0),
            Position::Center => (0, 0),
            Position::CenterRight => (78, 0),
            Position::BottomLeft => (-39, 45),
            Position::BottomRight => (39, 45),
            Position::Bottom => (0, 90),
        })
    }
}
