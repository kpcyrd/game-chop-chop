use crate::gfx;
use crate::timer::Timer;
use core::fmt::Debug;
use core::mem;
use embedded_graphics::{
    draw_target::DrawTarget,
    mono_font::MonoFont,
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::Rectangle,
    text::{Baseline, Text},
};

const NARRATOR_Y_OFFSET: i32 = 7;
const NARRATOR_X_OFFSET: i32 = 0;
const NARRATOR_AREA_WIDTH: u32 = gfx::UDISPLAY_WIDTH;

const LINE_HEIGHT: i32 = 8;
const BACKGROUND_HEIGHT: u32 = 9;
const BACKGROUND_Y_PADDING: i32 = 1;

#[derive(Clone)]
pub struct Narrator {
    text: &'static [&'static str],
    delay: Timer,
    scroll: Timer,
}

impl Narrator {
    pub const fn level0() -> Self {
        Narrator {
            text: &["Oh no,", "it's stuck!", " ", "Can you help us?", "._."],
            delay: Timer::new(3),
            scroll: Timer::infinite(),
        }
    }

    pub fn button_pressed(mut self) -> Option<Self> {
        if !self.started() {
            Some(self)
        } else if self.done() {
            None
        } else {
            self.reveal();
            Some(self)
        }
    }

    fn length(&self) -> usize {
        self.text
            .iter()
            .fold(0, |acc, text| acc.saturating_add(text.len()))
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
        if !self.started() {
            return;
        }

        let mut budget = self.scroll.get() as usize;

        let style = gfx::TEXT_STYLE;
        for (num, text) in self.text.iter().enumerate() {
            let y = NARRATOR_Y_OFFSET + (num as i32 * LINE_HEIGHT);

            let text = if let Some(remaining) = budget.checked_sub(text.len()) {
                // text is fitting in entirety
                budget = remaining;
                text
            } else {
                // truncate text to scroll position
                let idx = mem::take(&mut budget);
                &text[..idx]
            };

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

            // reached end of current scroll position
            if budget == 0 {
                break;
            }
        }
    }

    #[inline]
    pub const fn tick(&mut self) {
        if self.started() {
            self.scroll.tick();
        } else {
            self.delay.tick();
        }
    }

    #[inline]
    pub const fn started(&self) -> bool {
        self.delay.is_due()
    }

    #[inline]
    pub const fn reveal(&mut self) {
        self.scroll.set_due();
    }

    #[inline]
    pub fn done(&self) -> bool {
        self.scroll.get() as usize >= self.length()
    }
}
