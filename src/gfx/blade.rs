#![warn(unused)] // TODO

use crate::game;
use crate::gfx;
use crate::timer::Timer;
use core::cmp;
use embedded_graphics::prelude::*;

pub const HEIGHT: u32 = 18;
pub const WIDTH: u32 = 32;
pub const ANGLE: u32 = 5;
pub const SHARP: i32 = 1;
pub const PADDING: i32 = 2;

pub const X_OFFSET: i32 =
    gfx::DISPLAY_WIDTH - ((game::NUM_LANES - 1) * game::LANE_WIDTH) as i32 - PADDING;
pub const TOP_SPEED: u8 = 4;

static_assertions::const_assert_eq!(X_OFFSET, 20);

pub struct Blade {
    bottom_right: Point,
    speed: Timer,
}

impl Blade {
    pub const fn new() -> Self {
        Blade {
            bottom_right: Point::new(X_OFFSET, 0),
            speed: Timer::new(TOP_SPEED),
        }
    }

    pub fn move_towards(&mut self, height: i32) -> bool {
        if self.bottom_right.y == height {
            self.speed.reset();
            return true;
        }

        let speed = cmp::min(self.speed.get(), TOP_SPEED) as i32;
        let next_height = cmp::min(self.bottom_right.y + speed, height);
        self.speed.tick();
        self.bottom_right = Point::new(X_OFFSET, next_height);
        false
    }

    pub fn points(&self) -> [Point; 7] {
        let bottom_right = self.bottom_right;
        [
            bottom_right,
            bottom_right - Point::new(0, HEIGHT as i32),
            bottom_right - Point::new(WIDTH as i32, HEIGHT as i32),
            bottom_right - Point::new(WIDTH as i32, ANGLE as i32),
            bottom_right,
            // sharp edge
            bottom_right - Point::new(0, SHARP),
            bottom_right - Point::new(WIDTH as i32, ANGLE as i32 + SHARP),
        ]
    }

    pub fn is_off_screen(&self) -> bool {
        self.bottom_right.y - HEIGHT as i32 > gfx::DISPLAY_HEIGHT
    }
}
