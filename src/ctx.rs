use crate::game::Game;
use crate::gameover::{Decision, Gameover};
use crate::intro::Intro;
use core::fmt::Debug;
use embedded_graphics::{draw_target::DrawTarget, pixelcolor::BinaryColor};

pub enum Context {
    Intro(Intro),
    Game(Game),
    Gameover(Gameover),
}

impl Context {
    pub const fn new() -> Self {
        // Context::Intro(Intro::new())
        Context::Gameover(Gameover::new(1337))
    }

    fn start_game(&mut self) {
        let mut game = Game::new();
        game.add_obstacle_at_row(8);
        game.add_obstacle_at_row(14);
        *self = Self::Game(game);
    }

    pub fn button_up(&mut self) {
        match self {
            Self::Intro(_intro) => (),
            Self::Game(game) => game.button_up(),
            Self::Gameover(gameover) => gameover.button_up(),
        }
    }

    pub fn button_down(&mut self) {
        match self {
            Self::Intro(_intro) => (),
            Self::Game(game) => game.button_down(),
            Self::Gameover(gameover) => gameover.button_down(),
        }
    }

    pub fn button_right(&mut self) {
        match self {
            Self::Intro(_intro) => (),
            Self::Game(game) => game.button_right(),
            Self::Gameover(gameover) => gameover.button_right(),
        }
    }

    pub fn button_left(&mut self) {
        match self {
            Self::Intro(_intro) => (),
            Self::Game(game) => game.button_left(),
            Self::Gameover(_gameover) => (),
        }
    }

    pub fn button_center(&mut self) {
        match self {
            Self::Intro(intro) => intro.button_center(),
            Self::Game(game) => game.button_center(),
            Self::Gameover(gameover) => gameover.button_center(),
        }
    }

    pub fn tick(&mut self) {
        match self {
            Self::Intro(intro) => {
                if intro.start {
                    self.start_game();
                }
            }
            Self::Game(game) => {
                game.tick();
                // TODO: check for game over
                // TODO: check for next level condition
            }
            Self::Gameover(gameover) => match gameover.decision() {
                Some(Decision::Quit) => {
                    *self = Self::Intro(Intro::new());
                }
                Some(Decision::Restart) => self.start_game(),
                None => (),
            },
        };
    }

    pub fn render<D: DrawTarget<Color = BinaryColor>>(&self, display: &mut D)
    where
        <D as DrawTarget>::Error: Debug,
    {
        match self {
            Self::Intro(intro) => intro.render(display),
            Self::Game(game) => game.render(display),
            Self::Gameover(gameover) => gameover.render(display),
        }
    }
}
