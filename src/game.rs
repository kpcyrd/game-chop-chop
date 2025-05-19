use crate::gfx;
use crate::gfx::blade::Blade;
use crate::gfx::tile::Tile;
use crate::pieces::{self, Piece};
use crate::timer::Timer;
use core::fmt::Debug;
use embedded_graphics::{
    draw_target::DrawTarget, pixelcolor::BinaryColor, prelude::*, primitives::Polyline, text::Text,
};

const MIN_LANE: u32 = 2;
pub const NUM_LANES: u32 = 8;
const NUM_ROWS: u32 = gfx::UDISPLAY_HEIGHT / LANE_WIDTH;
pub const LANE_WIDTH: u32 = 6;
const LANE_OFFSET: Point = Point::new(gfx::DISPLAY_WIDTH - (LANE_WIDTH * NUM_LANES) as i32, 0);

const DROP_SPEED: u8 = 1;
const INITIAL_DROP_POSITION: i32 = -(4 * LANE_WIDTH as i32);
const INITIAL_LANE: u32 = MIN_LANE + 2;

static_assertions::const_assert_eq!(NUM_ROWS, 21);
static_assertions::const_assert!(INITIAL_LANE + 4 <= NUM_LANES);

#[derive(Clone)]
pub struct Game {
    blade: Blade,
    lane: u32,
    piece: pieces::Grid,
    drop: i32,
    drop_timer: Timer,
    drop_speed: i32,
    lanes: [[Option<Tile>; NUM_ROWS as usize]; NUM_LANES as usize],
}

impl Game {
    pub const fn new() -> Self {
        Game {
            blade: Blade::new(),
            lane: INITIAL_LANE,
            piece: Piece::T.into_grid(),
            drop: INITIAL_DROP_POSITION,
            drop_timer: Timer::new(DROP_SPEED),
            drop_speed: 1,
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
        self.try_to(|game| {
            game.drop_speed = i32::MAX;
        });
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

    pub fn tick(&mut self) {
        // blade fall animation
        if !self.blade.move_towards(self.next_obstacle_target_height()) {
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
                self.persist_piece();
                self.spawn_next_piece();
                break;
            }
        }

        // check completed rows
        self.check_completed_rows();
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

    fn persist_piece(&mut self) {
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
                let Some(lane) = self.lanes.get_mut(x) else {
                    continue;
                };
                let Some(tile) = lane.get_mut(y) else {
                    continue;
                };
                *tile = Some(Tile { wall: false });
            }
        }
    }

    pub fn spawn_next_piece(&mut self) {
        let next_piece = if self.piece.piece == Piece::T {
            Piece::S
        } else {
            Piece::T
        };

        self.piece = next_piece.into_grid();

        self.lane = INITIAL_LANE;
        self.drop = -(self.piece.lowest_point() as i32 * LANE_WIDTH as i32);
        self.drop_speed = 1; // TODO: this may get faster over time
    }

    pub fn add_obstacle_at_row(&mut self, row: usize) {
        self.lanes[0][row] = Some(Tile { wall: false });
        self.lanes[1][row] = Some(Tile { wall: false });
    }

    pub fn next_obstacle_target_height(&self) -> i32 {
        for (idx, tile) in self.lanes[0].iter().enumerate() {
            if tile.is_some() {
                return (idx as i32 * LANE_WIDTH as i32) - gfx::blade::PADDING;
            }
        }
        i32::MAX
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

        // render text on success
        if self.blade.is_off_screen() {
            let y = gfx::text_vertical_center(gfx::DISPLAY_HEIGHT, gfx::TEXT_STYLE.font);
            Text::new("yey!", Point::new(4, y), gfx::TEXT_STYLE)
                .draw(display)
                .unwrap();
        }
    }
}
