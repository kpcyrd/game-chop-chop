use crate::gfx;
use crate::gfx::blade::Blade;
use crate::gfx::tile::Tile;
use crate::narrator::Narrator;
use crate::pieces::{self, Piece};
use crate::random::Random;
use crate::timer::Timer;
use core::fmt::Debug;
use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Line, Polyline},
    text::Text,
};
use rand_core::RngCore;

const MIN_LANE: u32 = 2;
pub const NUM_LANES: u32 = 8;
const NUM_ROWS: u32 = gfx::UDISPLAY_HEIGHT / LANE_WIDTH;
pub const LANE_WIDTH: u32 = 6;
const LANE_OFFSET: Point = Point::new(
    gfx::DISPLAY_WIDTH - (LANE_WIDTH * NUM_LANES) as i32 - RIGHT_BORDER,
    0,
);
pub const RIGHT_BORDER: i32 = 1;

const DROP_SPEED: u8 = 1;
const INITIAL_DROP_POSITION: i32 = -(4 * LANE_WIDTH as i32);
const INITIAL_LANE: u32 = MIN_LANE + 2;

const NEXT_LEVEL_DELAY: u8 = 18;
const GAME_OVER_DELAY: u8 = 3;

static_assertions::const_assert_eq!(NUM_ROWS, 21);
static_assertions::const_assert!(INITIAL_LANE + 4 <= NUM_LANES);

#[derive(Clone, Copy)]
pub enum SwitchTo {
    NextLevel(u32),
    GameOver(u32),
}

#[derive(Clone)]
pub struct Game {
    level: u32,
    blade: Blade,
    lane: u32,
    piece: pieces::Grid,
    drop: i32,
    drop_timer: Timer,
    drop_speed: i32,
    narrator: Option<Narrator>,
    lanes: [[Option<Tile>; NUM_ROWS as usize]; NUM_LANES as usize],
    transiton: Option<(SwitchTo, Timer)>,
}

impl Game {
    pub const fn new(level: u32) -> Self {
        let narrator = match level {
            0 => Some(Narrator::level0()),
            _ => None,
        };

        Game {
            level,
            blade: Blade::new(),
            lane: INITIAL_LANE,
            piece: Piece::T.into_grid(),
            drop: INITIAL_DROP_POSITION,
            drop_timer: Timer::new(DROP_SPEED),
            drop_speed: 1,
            narrator,
            lanes: [
                [None; NUM_ROWS as usize],
                [Some(Tile { wall: true }); NUM_ROWS as usize],
                [None; NUM_ROWS as usize],
                [None; NUM_ROWS as usize],
                [None; NUM_ROWS as usize],
                [None; NUM_ROWS as usize],
                [None; NUM_ROWS as usize],
                [None; NUM_ROWS as usize],
                /*
                [Some(Tile { wall: false }); NUM_ROWS as usize],
                [Some(Tile { wall: false }); NUM_ROWS as usize],
                [Some(Tile { wall: false }); NUM_ROWS as usize],
                [Some(Tile { wall: false }); NUM_ROWS as usize],
                [Some(Tile { wall: false }); NUM_ROWS as usize],
                [Some(Tile { wall: false }); NUM_ROWS as usize],
                */
            ],
            transiton: None,
        }
    }

    fn try_to<F: Fn(&mut Self)>(&mut self, update: F) -> bool {
        let mut next = self.clone();
        update(&mut next);
        if !next.collides() {
            *self = next;
            true
        } else {
            false
        }
    }

    fn collides(&self) -> bool {
        // check left wall
        if self.lane + self.piece.padding_left() < MIN_LANE {
            return true;
        }

        // check right wall
        if self.lane + pieces::GRID_WIDTH - self.piece.padding_right() > NUM_LANES {
            return true;
        }

        // check tiles
        let offset_y = (self.drop / LANE_WIDTH as i32) + 1;
        for (x, lane) in self.piece.tiles().iter().enumerate() {
            let x = self.lane as usize + x;
            for (y, tile) in lane.iter().enumerate() {
                // if there's nothing in the piece grid
                if !tile {
                    continue;
                }

                // only consider piece tiles that are visible
                let Ok(y) = usize::try_from(offset_y + y as i32) else {
                    continue;
                };

                // check for collision
                let Some(lane) = self.lanes.get(x) else {
                    return true;
                };
                let Some(tile) = lane.get(y) else {
                    return true;
                };
                if tile.is_some() {
                    return true;
                }
            }
        }

        // everything is fine
        false
    }

    pub fn button_up(&mut self) {
        self.try_to(|game| {
            game.piece.rotate();
        });
    }

    pub fn button_down(&mut self) {
        if let Some(narrator) = self.narrator.take() {
            self.narrator = narrator.button_pressed();
        } else {
            self.try_to(|game| {
                game.drop_speed = i32::MAX;
            });
        }
    }

    pub fn button_right(&mut self) {
        self.try_to(|game| {
            game.lane = game.lane.saturating_add(1);
        });
    }

    pub fn button_left(&mut self) {
        self.try_to(|game| {
            game.lane = game.lane.saturating_sub(1);
        });
    }

    #[inline(always)]
    pub fn button_center(&mut self) {
        self.button_down();
    }

    pub fn tick<R: RngCore>(&mut self, random: &mut Random<R>) {
        // next-level condition and switch
        if let Some((_, timer)) = &mut self.transiton {
            timer.tick();
            return;
        } else if self.blade.is_off_screen() {
            let next_level = self.level.saturating_add(1);
            self.switch_to(SwitchTo::NextLevel(next_level));
        }

        // blade fall animation
        let (obstable, obstacle_height) = self.next_obstacle();
        if !self.blade.move_towards(obstacle_height) {
            return;
        }
        if let Some(row) = obstable {
            self.blade_hits_row(row);
        }

        // display narrator (if any)
        if let Some(narrator) = &mut self.narrator {
            narrator.tick();
            return;
        }

        // increase piece drop progression
        if !self.drop_timer.step() {
            return;
        }

        // collision detection
        for _ in 0..self.drop_speed {
            let collision = !self.try_to(|game| {
                game.drop = game.drop.saturating_add(1);
            });

            if collision {
                // next piece
                if self.persist_piece() {
                    self.spawn_next_piece(random);
                    break;
                } else {
                    // game over
                    self.switch_to(SwitchTo::GameOver(self.level));
                    return;
                }
            }
        }

        // check completed rows
        self.check_completed_rows();
    }

    fn switch_to(&mut self, target: SwitchTo) {
        self.transiton.get_or_insert_with(|| {
            (
                target,
                Timer::new(match target {
                    SwitchTo::NextLevel(_) => NEXT_LEVEL_DELAY,
                    SwitchTo::GameOver(_) => GAME_OVER_DELAY,
                }),
            )
        });
    }

    pub fn transition(&self) -> Option<SwitchTo> {
        let (target, timer) = self.transiton.as_ref()?;
        timer.is_due().then_some(*target)
    }

    fn check_completed_rows(&mut self) {
        for y in 0..NUM_ROWS {
            let y = y as usize;

            let complete = self.lanes.iter().enumerate().all(|(x, lane)| {
                if x < MIN_LANE as usize {
                    return true;
                }
                let Some(tile) = lane.get(y) else {
                    return false;
                };
                let Some(_) = tile else {
                    return false;
                };
                true
            });
            if !complete {
                continue;
            }

            self.clear_row(y);
            self.shift_previous_rows(y);
        }
    }

    fn shift_previous_rows(&mut self, y: usize) {
        for y in (0..y).rev() {
            for (x, lane) in self.lanes.iter_mut().enumerate() {
                if x >= MIN_LANE as usize {
                    lane[y + 1] = lane[y];
                }
            }
        }
        for (x, lane) in self.lanes.iter_mut().enumerate() {
            if x >= MIN_LANE as usize {
                lane[0] = None;
            }
        }
    }

    fn clear_row(&mut self, y: usize) {
        for lane in &mut self.lanes {
            let Some(slot) = lane.get_mut(y) else {
                continue;
            };
            if *slot == Some(Tile { wall: false }) {
                *slot = None;
            }
        }
    }

    fn persist_piece(&mut self) -> bool {
        let mut gameover = false;
        let offset_y = (self.drop / LANE_WIDTH as i32) + 1;
        for (x, lane) in self.piece.tiles().iter().enumerate() {
            let x = self.lane as usize + x;
            for (y, tile) in lane.iter().enumerate() {
                // if there's nothing in the piece grid
                if !tile {
                    continue;
                }

                // check gameover condition
                let y = offset_y + y as i32;
                if y <= 0 {
                    gameover = true;
                }

                // only consider piece tiles that are visible
                let Ok(y) = usize::try_from(y) else {
                    continue;
                };

                // check for collision
                let Some(lane) = self.lanes.get_mut(x) else {
                    continue;
                };
                let Some(tile) = lane.get_mut(y) else {
                    continue;
                };
                *tile = Some(Tile { wall: false });
            }
        }
        !gameover
    }

    pub fn spawn_next_piece<R: RngCore>(&mut self, random: &mut Random<R>) {
        let next_piece = {
            let mut current = Some(self.piece.piece);
            loop {
                let random = (random.squeeze() as u8) % 8;
                let mut next = match random {
                    0 => None, // Removed: Some(Piece::O),
                    1 => Some(Piece::I),
                    2 => Some(Piece::J),
                    3 => Some(Piece::L),
                    4 => Some(Piece::T),
                    5 => Some(Piece::S),
                    6 => Some(Piece::Z),
                    _ => None,
                };
                // reduce the chance of a duplicate piece, but not impossible
                if next == current.take() {
                    next = None;
                }
                if let Some(piece) = next {
                    break piece;
                }
            }
        };

        self.piece = next_piece.into_grid();

        self.lane = INITIAL_LANE;
        self.drop = -(self.piece.lowest_point() as i32 * LANE_WIDTH as i32);
        self.drop_speed = 1; // TODO: this may get faster over time
    }

    /// lowest possible number can be 1
    /// good upper bound is 15
    pub fn add_obstacle_at_row(&mut self, row: u32) {
        let row = NUM_ROWS.saturating_sub(row) as usize;
        self.lanes[0][row] = Some(Tile { wall: false });
        self.lanes[1][row] = Some(Tile { wall: false });
    }

    pub fn add_tough_obstacle_at_row(&mut self, row: u32) {
        let row = NUM_ROWS.saturating_sub(row) as usize;
        self.lanes[0][row] = Some(Tile { wall: true });
        self.lanes[1][row] = Some(Tile { wall: true });
    }

    pub fn next_obstacle(&self) -> (Option<usize>, i32) {
        for (idx, tile) in self.lanes[0].iter().enumerate() {
            if tile.is_some() {
                let height = (idx as i32 * LANE_WIDTH as i32) - gfx::blade::PADDING;
                return (Some(idx), height);
            }
        }
        (None, i32::MAX)
    }

    pub fn blade_hits_row(&mut self, row: usize) {
        for idx in [0, 1] {
            let tile = &mut self.lanes[idx][row];
            if let Some(tile) = tile {
                tile.wall = false;
            }
        }
    }

    pub fn render<D: DrawTarget<Color = BinaryColor>>(&self, display: &mut D)
    where
        <D as DrawTarget>::Error: Debug,
    {
        // render game
        for (column, lane) in self.lanes.iter().enumerate() {
            for (row, tile) in lane.iter().enumerate() {
                let point = LANE_OFFSET
                    + Point::new(
                        column as i32 * LANE_WIDTH as i32,
                        row as i32 * LANE_WIDTH as i32,
                    );

                let Some(tile) = tile else { continue };
                tile.render(display, point);
            }
        }

        // render current piece
        self.piece.render(
            display,
            LANE_OFFSET + Point::new((LANE_WIDTH * self.lane) as i32, self.drop),
        );

        // render blade
        Polyline::new(&self.blade.points())
            .into_styled(gfx::WHITE_LINE)
            .draw(display)
            .unwrap();

        // render right border
        Line::new(
            Point::new(gfx::DISPLAY_WIDTH - RIGHT_BORDER, 0),
            Point::new(
                gfx::DISPLAY_WIDTH - RIGHT_BORDER,
                (NUM_ROWS * LANE_WIDTH) as i32,
            ),
        )
        .into_styled(gfx::WHITE_LINE)
        .draw(display)
        .unwrap();

        // render narrator
        if let Some(narrator) = &self.narrator {
            narrator.render(display);
        }

        // render text on success
        if self.blade.is_off_screen() {
            let y = gfx::text_vertical_center(gfx::DISPLAY_HEIGHT, gfx::TEXT_STYLE.font);
            Text::new("yey!", Point::new(3, y), gfx::TEXT_STYLE)
                .draw(display)
                .unwrap();
        }
    }
}
