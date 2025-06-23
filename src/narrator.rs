use crate::gfx;
use crate::timer::Timer;
use core::fmt::Debug;
use embedded_graphics::{
    draw_target::DrawTarget,
    mono_font::MonoFont,
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::Rectangle,
    text::{Baseline, Text},
};

const NARRATOR_Y_OFFSET: i32 = 7;
// const NARRATOR_X_OFFSET: i32 = gfx::DISPLAY_WIDTH - NARRATOR_AREA_WIDTH as i32 - game::RIGHT_BORDER;
// const NARRATOR_AREA_WIDTH: u32 = game::LANE_WIDTH * game::NUM_LANES;
const NARRATOR_X_OFFSET: i32 = 0;
const NARRATOR_AREA_WIDTH: u32 = gfx::UDISPLAY_WIDTH;

const LINE_HEIGHT: i32 = 8;
const BACKGROUND_HEIGHT: u32 = 9;
const BACKGROUND_Y_PADDING: i32 = 1;

#[derive(Clone)]
pub struct Narrator {
    text: &'static [&'static str],
    delay: Timer,
}

impl Narrator {
    pub const fn level0() -> Self {
        Narrator {
            text: &["Oh no,", "it's stuck!", "", "Can you help us?", "._."],
            delay: Timer::new(6),
        }
    }

    const fn center(y: i32, text: &str, font: &MonoFont) -> Point {
        Point::new(
            NARRATOR_X_OFFSET + gfx::text_align_center(text, NARRATOR_AREA_WIDTH as i32, font),
            y,
        )
    }

    pub fn render<D: DrawTarget<Color = BinaryColor>>(&self, display: &mut D)
    where
        <D as DrawTarget>::Error: Debug,
    {
        if !self.visible() {
            return;
        }

        let style = gfx::TEXT_STYLE;
        for (num, text) in self.text.iter().enumerate() {
            let y = NARRATOR_Y_OFFSET + (num as i32 * LINE_HEIGHT);

            // draw black background
            Rectangle::new(
                Point::new(0, y - BACKGROUND_Y_PADDING),
                Size::new(gfx::UDISPLAY_WIDTH, BACKGROUND_HEIGHT),
            )
            .into_styled(gfx::BLACK)
            .draw(display)
            .unwrap();

            // render text
            Text::with_baseline(
                text,
                Self::center(y, text, style.font),
                style,
                Baseline::Top,
            )
            .draw(display)
            .unwrap();
        }
    }

    #[inline]
    pub const fn tick(&mut self) {
        self.delay.tick();
    }

    #[inline]
    pub const fn visible(&self) -> bool {
        self.delay.is_due()
    }
}
