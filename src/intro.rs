use crate::gfx;
use core::fmt::Debug;
use embedded_graphics::{
    draw_target::DrawTarget,
    image::{Image, ImageRaw},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};

const INTRO: ImageRaw<BinaryColor> = ImageRaw::new(include_bytes!("../video/intro.raw"), 37);
const SIG: ImageRaw<BinaryColor> = ImageRaw::new(include_bytes!("../video/sig.raw"), 35);

const LOGO_Y_POSITION: i32 = 6;
const TEXT_Y_POSITION: i32 = 79;
const SIG_BOTTOM_PADDING: i32 = 3;

pub struct Intro {
    transition: bool,
}

impl Intro {
    pub const fn new() -> Self {
        Intro { transition: false }
    }

    pub fn button_center(&mut self) {
        self.transition = true;
    }

    pub fn render<D: DrawTarget<Color = BinaryColor>>(&self, display: &mut D)
    where
        <D as DrawTarget>::Error: Debug,
    {
        self.render_logo(display);
        self.render_text(display);
        self.render_sig(display);
    }

    #[inline(always)]
    fn render_logo<D: DrawTarget<Color = BinaryColor>>(&self, display: &mut D)
    where
        <D as DrawTarget>::Error: Debug,
    {
        let x = gfx::centered(gfx::DISPLAY_WIDTH, INTRO.size().width);
        let point = Point::new(x, LOGO_Y_POSITION);
        Image::new(&INTRO, point).draw(display).unwrap();
    }

    #[inline(always)]
    fn render_text<D: DrawTarget<Color = BinaryColor>>(&self, display: &mut D)
    where
        <D as DrawTarget>::Error: Debug,
    {
        let style = gfx::TEXT_STYLE;

        for (num, text) in ["Designed and", "programmed by"].iter().enumerate() {
            let y = TEXT_Y_POSITION + (num as i32 * 9);
            Text::with_baseline(
                text,
                Point::new(
                    gfx::text_align_center(text, gfx::DISPLAY_WIDTH, style.font),
                    y,
                ),
                style,
                Baseline::Top,
            )
            .draw(display)
            .unwrap();
        }
    }

    #[inline(always)]
    fn render_sig<D: DrawTarget<Color = BinaryColor>>(&self, display: &mut D)
    where
        <D as DrawTarget>::Error: Debug,
    {
        let x = gfx::centered(gfx::DISPLAY_WIDTH, SIG.size().width);
        let y = gfx::DISPLAY_HEIGHT - (SIG.size().height as i32) - SIG_BOTTOM_PADDING;
        let point = Point::new(x, y);
        Image::new(&SIG, point).draw(display).unwrap();
    }
}
