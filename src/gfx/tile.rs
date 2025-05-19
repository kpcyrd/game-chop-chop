#![warn(unused)] // TODO

use crate::game::LANE_WIDTH;
use crate::gfx;
use core::fmt::Debug;
use embedded_graphics::{
    draw_target::DrawTarget, pixelcolor::BinaryColor, prelude::*, primitives::Rectangle,
};

#[derive(Clone, Copy, PartialEq)]
pub struct Tile {
    pub wall: bool,
}

impl Tile {
    pub fn render<D: DrawTarget<Color = BinaryColor>>(&self, display: &mut D, point: Point)
    where
        <D as DrawTarget>::Error: Debug,
    {
        Rectangle::new(point, Size::new(LANE_WIDTH, LANE_WIDTH))
            .into_styled(gfx::WHITE)
            .draw(display)
            .unwrap();

        if !self.wall {
            Rectangle::new(
                point + Point::new(1, 1),
                Size::new(LANE_WIDTH - 2, LANE_WIDTH - 2),
            )
            .into_styled(gfx::BLACK_LINE)
            .draw(display)
            .unwrap();
        }
    }
}
