mod position;

use core::array;

use eg_seven_segment::SevenSegmentStyleBuilder;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyleBuilder, Rectangle, RoundedRectangle};
use embedded_graphics::text::Text;
use heapless::String;

pub use position::Position;

pub struct Lcd199 {
    position: Position,
    value: Option<i32>,
    frame_fill_color: Rgb565,
    frame_stroke_color: Rgb565,
    frame_stroke_width: u32,
    within_range_color: [Rgb565; 26],
    out_of_range_color: [Rgb565; 26],
}

impl Lcd199 {
    pub fn new(position: Position) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }

    pub fn with_value(position: Position, value: i32) -> Self {
        Self {
            position,
            value: Some(value),
            ..Default::default()
        }
    }
}

impl Default for Lcd199 {
    fn default() -> Self {
        Self {
            position: Position::Center,
            value: None,
            frame_fill_color: Rgb565::BLACK,
            frame_stroke_color: Rgb565::new(18, 30, 8),
            frame_stroke_width: 1,
            within_range_color: array::from_fn(|i| Rgb565::new(20, 38 + i as u8, 18)),
            out_of_range_color: array::from_fn(|i| Rgb565::new(20, 37 - i as u8, 18)),
        }
    }
}

impl Drawable for Lcd199 {
    type Color = Rgb565;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let delta = Point::from(self.position);
        let style = PrimitiveStyleBuilder::new()
            .fill_color(self.frame_fill_color)
            .stroke_color(self.frame_stroke_color)
            .stroke_width(self.frame_stroke_width)
            .build();

        let rectangle = Rectangle::new(Point::new(89, 97) + delta, Size::new(62, 46));

        RoundedRectangle::with_equal_corners(rectangle, Size::new_equal(8))
            .into_styled(style)
            .draw(target)?;

        let text: String<7> = String::from_iter([
            match self.value {
                Some(i32::MIN..=99) => ' ',
                Some(100..=199) => '1',
                Some(200..=i32::MAX) => ' ',
                None => ' ',
            },
            match self.value {
                Some(i32::MIN..=-10) => '_',
                Some(-9..=-1) => '-',
                Some(0..=9) => ' ',
                Some(i @ 10..=199) => char::from_digit((i as u32 % 100) / 10, 10).unwrap_or('?'),
                Some(200..=i32::MAX) => '\u{E040}',
                None => '-',
            },
            match self.value {
                Some(i32::MIN..=-10) => '_',
                Some(i @ -9..=-1) => char::from_digit(-i as u32, 10).unwrap_or('?'),
                Some(i @ 0..=199) => char::from_digit(i as u32 % 10, 10).unwrap_or('?'),
                Some(200..=i32::MAX) => '\u{E040}',
                None => '-',
            },
        ]);

        let segment_color = match self.value {
            Some(i32::MIN..=-1) => self.out_of_range_color[25],
            Some(i @ 0..=24) => self.out_of_range_color[24 - (i as usize)],
            Some(i @ 25..=100) => self.within_range_color[(i as usize - 25) / 3],
            Some(101..=i32::MAX) => self.out_of_range_color[25],
            None => self.within_range_color[0],
        };

        let character_style = SevenSegmentStyleBuilder::new()
            .digit_size(Size::new(20, 36))
            .digit_spacing(4)
            .segment_width(4)
            .segment_color(segment_color)
            .build();

        Text::new(text.as_str(), Point::new(78, 137) + delta, character_style).draw(target)?;

        Ok(())
    }
}
