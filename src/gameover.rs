use crate::gfx;
use core::fmt::Debug;
use embedded_graphics::{
    draw_target::DrawTarget,
    mono_font::MonoTextStyle,
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};

const GAMEOVER_Y_OFFSET: i32 = 20;
const SCORE_Y_OFFSET: i32 = 50;
const MENU_Y_OFFSET: i32 = 95;
const QUIT_Y_OFFSET: i32 = MENU_Y_OFFSET;
const RESTART_Y_OFFSET: i32 = MENU_Y_OFFSET + 10;
const CURSOR_X_OFFSET: i32 = 2;

#[derive(Clone, Copy)]
pub enum Decision {
    Quit,
    Restart,
}

impl Decision {
    pub fn toggle(&mut self) {
        *self = match self {
            Decision::Quit => Decision::Restart,
            Decision::Restart => Decision::Quit,
        };
    }
}

pub struct Gameover {
    score: u32,
    decision: Decision,
    confirmed: bool,
}

impl Gameover {
    pub const fn new(score: u32) -> Self {
        Self {
            score,
            decision: Decision::Quit,
            confirmed: false,
        }
    }

    pub fn decision(&self) -> Option<Decision> {
        self.confirmed.then_some(self.decision)
    }

    #[inline(always)]
    pub fn button_up(&mut self) {
        self.decision.toggle();
    }

    #[inline(always)]
    pub fn button_down(&mut self) {
        self.decision.toggle();
    }

    #[inline(always)]
    pub fn button_right(&mut self) {
        self.button_center();
    }

    /// confirm selection
    pub fn button_center(&mut self) {
        self.confirmed = true;
    }

    pub fn render_centered<D: DrawTarget<Color = BinaryColor>>(
        text: &str,
        y: i32,
        style: MonoTextStyle<BinaryColor>,
        display: &mut D,
    ) where
        <D as DrawTarget>::Error: Debug,
    {
        let x = gfx::text_align_center(text, gfx::DISPLAY_WIDTH, style.font);
        Text::with_baseline(text, Point::new(x, y), style, Baseline::Top)
            .draw(display)
            .unwrap();
    }

    pub fn render<D: DrawTarget<Color = BinaryColor>>(&self, display: &mut D)
    where
        <D as DrawTarget>::Error: Debug,
    {
        // render game over text
        Self::render_centered("Game over", GAMEOVER_Y_OFFSET, gfx::BIG_TEXT_STYLE, display);

        // render score
        Self::render_centered("Held", SCORE_Y_OFFSET, gfx::TEXT_STYLE, display);

        let mut buf = itoa::Buffer::new();
        let buf = buf.format(self.score);
        Self::render_centered(buf, SCORE_Y_OFFSET + 10, gfx::TEXT_STYLE, display);

        Self::render_centered(
            "CEOs accountable",
            SCORE_Y_OFFSET + 20,
            gfx::TEXT_STYLE,
            display,
        );

        // render options
        Text::with_baseline(
            "Return to 9-5",
            Point::new(10, QUIT_Y_OFFSET),
            gfx::TEXT_STYLE,
            Baseline::Top,
        )
        .draw(display)
        .unwrap();

        Text::with_baseline(
            "Try again",
            Point::new(10, RESTART_Y_OFFSET),
            gfx::TEXT_STYLE,
            Baseline::Top,
        )
        .draw(display)
        .unwrap();

        // render pointer
        let y = match self.decision {
            Decision::Quit => QUIT_Y_OFFSET,
            Decision::Restart => RESTART_Y_OFFSET,
        };
        Text::with_baseline(
            ">",
            Point::new(CURSOR_X_OFFSET, y),
            gfx::TEXT_STYLE,
            Baseline::Top,
        )
        .draw(display)
        .unwrap();
    }
}
