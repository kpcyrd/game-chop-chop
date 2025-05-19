use crate::Game;
use crate::intro::Intro;
use core::fmt::Debug;
use embedded_graphics::{draw_target::DrawTarget, pixelcolor::BinaryColor};

pub enum Context {
    Intro(Intro),
    Game(Game),
    Gameover,
}

impl Context {
    pub const fn new() -> Self {
        Context::Intro(Intro::new())
    }

    pub fn button_up(&mut self) {
        match self {
            Self::Intro(_intro) => (),
            Self::Game(game) => game.button_up(),
            Self::Gameover => (),
        }
    }

    pub fn button_down(&mut self) {
        match self {
            Self::Intro(_intro) => (),
            Self::Game(game) => game.button_down(),
            Self::Gameover => (),
        }
    }

    pub fn button_right(&mut self) {
        match self {
            Self::Intro(_intro) => (),
            Self::Game(game) => game.button_right(),
            Self::Gameover => (),
        }
    }

    pub fn button_left(&mut self) {
        match self {
            Self::Intro(_intro) => (),
            Self::Game(game) => game.button_left(),
            Self::Gameover => (),
        }
    }

    pub fn button_center(&mut self) {
        match self {
            Self::Intro(_intro) => {
                let mut game = Game::new();
                game.add_obstacle_at_row(8);
                game.add_obstacle_at_row(14);
                *self = Self::Game(game);
            }
            Self::Game(game) => game.button_center(),
            Self::Gameover => (),
        }
    }

    pub fn tick(&mut self) {
        match self {
            Self::Intro(_intro) => (),
            Self::Game(game) => game.tick(),
            Self::Gameover => (),
        }
    }

    pub fn render<D: DrawTarget<Color = BinaryColor>>(&self, display: &mut D)
    where
        <D as DrawTarget>::Error: Debug,
    {
        match self {
            Self::Intro(intro) => intro.render(display),
            Self::Game(game) => game.render(display),
            Self::Gameover => (),
        }
    }
}
