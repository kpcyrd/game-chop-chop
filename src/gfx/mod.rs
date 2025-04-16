#![allow(unused)] // TODO

pub mod blade;
pub mod tile;

use embedded_graphics::{
    mono_font::{MonoFont, MonoTextStyle, MonoTextStyleBuilder, ascii},
    pixelcolor::BinaryColor,
    primitives::PrimitiveStyle,
};

pub const DISPLAY_WIDTH: i32 = 64;
pub const DISPLAY_HEIGHT: i32 = 128;
pub const UDISPLAY_WIDTH: u32 = DISPLAY_WIDTH as u32;
pub const UDISPLAY_HEIGHT: u32 = DISPLAY_HEIGHT as u32;

pub const WHITE: PrimitiveStyle<BinaryColor> = PrimitiveStyle::with_fill(BinaryColor::On);
pub const BLACK: PrimitiveStyle<BinaryColor> = PrimitiveStyle::with_fill(BinaryColor::Off);

pub const WHITE_LINE: PrimitiveStyle<BinaryColor> = PrimitiveStyle::with_stroke(BinaryColor::On, 1);
pub const BLACK_LINE: PrimitiveStyle<BinaryColor> =
    PrimitiveStyle::with_stroke(BinaryColor::Off, 1);

pub const TEXT_STYLE: MonoTextStyle<BinaryColor> = MonoTextStyleBuilder::new()
    .font(&ascii::FONT_4X6)
    .text_color(BinaryColor::On)
    .build();

pub const fn centered(outer: i32, inner: u32) -> i32 {
    outer / 2 - (inner as i32 / 2)
}

pub const fn text_vertical_center(total: i32, font: &MonoFont) -> i32 {
    centered(total, font.character_size.height)
}
