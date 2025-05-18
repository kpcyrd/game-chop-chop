use crate::game::LANE_WIDTH;
use crate::gfx::tile::Tile;
use core::fmt::Debug;
use embedded_graphics::{draw_target::DrawTarget, pixelcolor::BinaryColor, prelude::*};

pub const GRID_WIDTH: u32 = 4;

/// tiles[x][y]
type Tiles = [[bool; 4]; GRID_WIDTH as usize];

#[allow(dead_code)] // TODO
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Piece {
    O,
    I,
    J,
    L,
    T,
    S,
    Z,
}

impl Piece {
    #[inline(always)]
    pub const fn into_grid(self) -> Grid {
        Grid::new(self)
    }

    const fn tiles(&self, tiles: &mut Tiles, rotation: Rotation) {
        match (self, rotation) {
            (Self::O, _) => {
                tiles[1][1] = true;
                tiles[1][2] = true;
                tiles[2][1] = true;
                tiles[2][2] = true;
            }

            (Self::I, Rotation::R0 | Rotation::R180) => {
                tiles[0][2] = true;
                tiles[1][2] = true;
                tiles[2][2] = true;
                tiles[3][2] = true;
            }
            (Self::I, Rotation::R90 | Rotation::R270) => {
                tiles[2][0] = true;
                tiles[2][1] = true;
                tiles[2][2] = true;
                tiles[2][3] = true;
            }

            (Self::J, Rotation::R0) => {
                tiles[0][1] = true;
                tiles[1][1] = true;
                tiles[2][1] = true;
                tiles[2][2] = true;
            }
            (Self::J, Rotation::R90) => {
                tiles[0][2] = true;
                tiles[1][0] = true;
                tiles[1][1] = true;
                tiles[1][2] = true;
            }
            (Self::J, Rotation::R180) => {
                tiles[0][0] = true;
                tiles[0][1] = true;
                tiles[1][1] = true;
                tiles[2][1] = true;
            }
            (Self::J, Rotation::R270) => {
                tiles[1][0] = true;
                tiles[1][1] = true;
                tiles[1][2] = true;
                tiles[2][0] = true;
            }

            (Self::L, Rotation::R0) => {
                tiles[0][1] = true;
                tiles[1][1] = true;
                tiles[2][1] = true;
                tiles[0][2] = true;
            }
            (Self::L, Rotation::R90) => {
                tiles[0][0] = true;
                tiles[1][0] = true;
                tiles[1][1] = true;
                tiles[1][2] = true;
            }
            (Self::L, Rotation::R180) => {
                tiles[2][0] = true;
                tiles[0][1] = true;
                tiles[1][1] = true;
                tiles[2][1] = true;
            }
            (Self::L, Rotation::R270) => {
                tiles[1][0] = true;
                tiles[1][1] = true;
                tiles[1][2] = true;
                tiles[2][2] = true;
            }

            (Self::T, Rotation::R0) => {
                tiles[0][1] = true;
                tiles[1][1] = true;
                tiles[2][1] = true;
                tiles[1][2] = true;
            }
            (Self::T, Rotation::R90) => {
                tiles[1][0] = true;
                tiles[0][1] = true;
                tiles[1][1] = true;
                tiles[1][2] = true;
            }
            (Self::T, Rotation::R180) => {
                tiles[1][0] = true;
                tiles[0][1] = true;
                tiles[1][1] = true;
                tiles[2][1] = true;
            }
            (Self::T, Rotation::R270) => {
                tiles[1][0] = true;
                tiles[1][1] = true;
                tiles[2][1] = true;
                tiles[1][2] = true;
            }

            (Self::S, Rotation::R0 | Rotation::R180) => {
                tiles[1][1] = true;
                tiles[2][1] = true;
                tiles[0][2] = true;
                tiles[1][2] = true;
            }
            (Self::S, Rotation::R90 | Rotation::R270) => {
                tiles[1][0] = true;
                tiles[1][1] = true;
                tiles[2][1] = true;
                tiles[2][2] = true;
            }

            (Self::Z, Rotation::R0 | Rotation::R180) => {
                tiles[0][1] = true;
                tiles[1][1] = true;
                tiles[1][2] = true;
                tiles[2][2] = true;
            }
            (Self::Z, Rotation::R90 | Rotation::R270) => {
                tiles[2][0] = true;
                tiles[1][1] = true;
                tiles[2][1] = true;
                tiles[2][1] = true;
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Rotation {
    R0,
    R90,
    R180,
    R270,
}

impl Rotation {
    pub fn rotate(&mut self) {
        *self = match self {
            Self::R0 => Self::R90,
            Self::R90 => Self::R180,
            Self::R180 => Self::R270,
            Self::R270 => Self::R0,
        }
    }
}

#[derive(Clone)]
pub struct Grid {
    tiles: Tiles,
    pub piece: Piece,
    rotation: Rotation,
}

impl Grid {
    pub const fn new(piece: Piece) -> Self {
        let mut tiles = [[false; 4]; 4];
        let rotation = Rotation::R0;
        piece.tiles(&mut tiles, rotation);
        Grid {
            tiles,
            piece,
            rotation: Rotation::R0,
        }
    }

    pub fn rotate(&mut self) {
        self.tiles = Default::default();
        self.rotation.rotate();
        self.piece.tiles(&mut self.tiles, self.rotation);
    }

    fn lowest_lane_point(lane: &[bool; 4]) -> Option<u8> {
        lane.iter()
            .enumerate()
            .flat_map(|(idx, v)| v.then_some(idx as u8))
            .max()
    }

    pub fn lowest_point(&self) -> u8 {
        self.tiles
            .iter()
            .flat_map(Self::lowest_lane_point)
            .max()
            .unwrap_or(0)
    }

    fn padding<I: Iterator<Item = [bool; 4]>>(iter: I) -> u32 {
        let mut padding = 0;
        for lane in iter {
            if lane.into_iter().any(|tile| tile) {
                break;
            }
            padding += 1;
        }
        padding
    }

    #[inline]
    pub fn padding_left(&self) -> u32 {
        Self::padding(self.tiles.into_iter())
    }

    #[inline]
    pub fn padding_right(&self) -> u32 {
        Self::padding(self.tiles.into_iter().rev())
    }

    pub fn collision_points(&self) -> [Option<u8>; 4] {
        let mut points = [None; 4];
        for (idx, lane) in self.tiles.iter().enumerate() {
            points[idx] = Self::lowest_lane_point(lane);
        }
        points
    }

    pub fn render<D: DrawTarget<Color = BinaryColor>>(&self, display: &mut D, point: Point)
    where
        <D as DrawTarget>::Error: Debug,
    {
        for (x, lane) in self.tiles.iter().enumerate() {
            for (y, tile) in lane.iter().enumerate() {
                if !tile {
                    continue;
                }

                Tile { wall: false }.render(
                    display,
                    point + Point::new(LANE_WIDTH as i32 * x as i32, LANE_WIDTH as i32 * y as i32),
                );
            }
        }
    }
}
