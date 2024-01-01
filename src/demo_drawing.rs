use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::primitives::{Circle, Line, Primitive, PrimitiveStyle};
use embedded_graphics::text::{Baseline, Text, TextStyleBuilder};
use embedded_graphics_core::Drawable;
use embedded_graphics_core::geometry::Point;
use embedded_graphics_core::pixelcolor::BinaryColor;
use embedded_graphics_core::prelude::Size;
use embedded_graphics_core::primitives::Rectangle;
use crate::epaper29::{E29Buffer};

pub fn demo_drawing_black(black_frame: &mut E29Buffer) {
    for i in 0..10 {
        let _ = Line::new(Point::new(0, i*10), Point::new(100-i*10, 0))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 2))
            .draw(black_frame);
    }
    let _ = Rectangle::new(Point::new(113,54), Size::new(12, 120))
        .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
        .draw(black_frame);
}

pub fn demo_drawing_red(red_frame: &mut E29Buffer) {
    for i in 0..10 {
        let _ = Circle::with_center(Point::new(64, 64+16*i), 80)
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(red_frame);
    }

    //write a line here
    let style = MonoTextStyleBuilder::new()
        .font(&embedded_graphics::mono_font::ascii::FONT_6X10)
        .background_color(BinaryColor::Off)
        .text_color(BinaryColor::On)
        .build();

    let text_style = TextStyleBuilder::new().baseline(Baseline::Top).build();
    let _ = Text::with_text_style("This is a demo ", Point::new(10, 266), style, text_style).draw(red_frame).unwrap();
    let _ = Text::with_text_style("Hello world! ", Point::new(10, 276), style, text_style).draw(red_frame).unwrap();
    let _ = Text::with_text_style("End of demo! ", Point::new(10, 286), style, text_style).draw(red_frame).unwrap();

}