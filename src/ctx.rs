use crate::game::{Game, SwitchTo};
use crate::gameover::{Decision, Gameover};
use crate::intro::Intro;
use core::fmt::Debug;
use embedded_graphics::{draw_target::DrawTarget, pixelcolor::BinaryColor};

#[allow(clippy::large_enum_variant)]
pub enum Context {
    Intro(Intro),
    Game(Game),
    Gameover(Gameover),
}

impl Context {
    pub const fn new() -> Self {
        // Context::Gameover(Gameover::new(1337))
        Context::Intro(Intro::new())
    }

    fn start_game(&mut self, level: u32) {
        let mut game = Game::new(level);
        // TODO: refactor this
        match level {
            0 => {
                game.add_obstacle_at_row(2);
            }
            1 => {
                game.add_obstacle_at_row(7);
            }
            _ => {
                game.add_obstacle_at_row(13);
                game.add_obstacle_at_row(7);
            }
        }
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
                    self.start_game(0);
                }
            }
            Self::Game(game) => {
                game.tick();
                // check for game over/next level
                match game.transition() {
                    Some(SwitchTo::NextLevel(level)) => {
                        self.start_game(level);
                    }
                    Some(SwitchTo::GameOver(level)) => {
                        *self = Self::Gameover(Gameover::new(level));
                    }
                    None => (),
                }
            }
            Self::Gameover(gameover) => match gameover.decision() {
                Some(Decision::Quit) => {
                    *self = Self::Intro(Intro::new());
                }
                Some(Decision::Restart) => self.start_game(0),
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
