use crate::gfx;
use crate::gfx::blade::Blade;
use crate::gfx::tile::Tile;
use crate::pieces::{self, Piece};
use crate::timer::Timer;
use core::cmp;
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

static_assertions::const_assert!(INITIAL_LANE + 4 <= NUM_LANES);

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

    pub fn button_up(&mut self) {
        self.piece.rotate();
    }

    pub fn button_down(&mut self) {
        self.drop_speed = i32::MAX;
    }

    pub fn button_right(&mut self) {
        self.lane = cmp::min(self.lane + 1, NUM_LANES - 1);
    }

    pub fn button_left(&mut self) {
        self.lane = cmp::max(self.lane.saturating_sub(1), MIN_LANE);
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

        // collision detection (TODO)
        let next_step = self.drop_speed;
        self.drop = self.drop.saturating_add(next_step);

        if self.drop / LANE_WIDTH as i32 >= NUM_ROWS as i32 - 1 {
            self.lanes[self.lane as usize][(NUM_ROWS - 1) as usize] = Some(Tile { wall: false });
            self.spawn_next_piece();
        }
    }

    pub fn spawn_next_piece(&mut self) {
        let next_piece = if self.piece.piece == Piece::T {
            Piece::S
        } else {
            Piece::T
        };

        self.piece = next_piece.into_grid();

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
