use crate::cos::cos;
use crate::sin::sin;
use crate::tan::tan;
use crate::wasm4::FRAMEBUFFER;
use crate::Direction;
use crate::BOARD;
// 160x160 pixels projection plane - 160 columns
// 60 degrees = pi/6 rad FOV
// angle_num diff per column (ray) = FOV / 160
// Centre of projection plane = (80,80)
//
// Walls are 4x128x4
// Camera height =65
// This code is FOV independent, but trig lookup tables are not!
const FOV: usize = 60; // degrees
const ANGLE_DIFF_DEGREES: f32 = FOV as f32 / 160.0;
const WALL_HEIGHT: usize = 8;
const WALL_SIZE: usize = 4; // x,z
const PROJECTION_DISTANCE: usize = 138; // Approximation of 80 half-width / tan(pi/6) i.e. 2**7
const SLICE_HEIGHT_CONST: usize = PROJECTION_DISTANCE * WALL_HEIGHT; // Divide this by distance to get actual height

#[derive(Copy, Clone)]
enum IntersectionKind {
    HorizontalGrid,
    VerticalGrid,
}

struct Intersection {
    kind: IntersectionKind,
    distance: f32,
    colour: u8,
}

fn calculate_distance(px: usize, py: usize, ax: usize, ay: usize, angle_num: usize) -> f32 {
    // let mut dist: f32;
    // if py != ay && sin(angle_num) != 0.0 {
    //     dist = py as f32 - ay as f32;
    //     if dist < 0.0 {
    //         dist *= -1.0;
    //     };
    //     dist /= sin(angle_num);
    // } else {
    //     dist = px as f32 - ax as f32;
    //     if dist < 0.0 {
    //         dist *= -1.0;
    //     };
    //     dist /= cos(angle_num);
    // }
    // dist
    ((px as f32 - ax as f32).powi(2) + (py as f32 - ay as f32).powi(2)).sqrt()
}

fn draw_cols<I>(angle_nums: I, grid_origin: (usize, usize))
where
    I: Iterator<Item = usize>,
{
    for (col, angle_num) in angle_nums.enumerate() {
        let intersection = find_intersection(grid_origin, angle_num);
        let height = (SLICE_HEIGHT_CONST as f32 / intersection.distance).floor() as usize;
        for row in 0..160 {
            let byte: usize = (40 * row) + (col / 4);
            let bit: u8 = col as u8 % 4;
            let bitshift: u8 = 2 * (3 - bit);
            let target_col: u8 = if (row as i32) > (80 - (height as i32 / 2))
                && (row as i32) < (80 + (height as i32 / 2))
            {
                // match intersection.kind {
                //     IntersectionKind::VerticalGrid => 1,
                //     IntersectionKind::HorizontalGrid => 2,
                // }
                intersection.colour
            } else {
                3
            };
            unsafe {
                (*FRAMEBUFFER)[byte] =
                    ((*FRAMEBUFFER)[byte] & (!((3 << bitshift) as u8))) | (target_col << bitshift);
            }
        }
    }
}

pub fn draw_3d(grid_origin: (usize, usize), dir: Direction) {
    use Direction::*;
    // FOV independent - depends on num columns
    match dir {
        North => {
            let angles = (160..320).rev();
            draw_cols(angles, grid_origin);
        }
        South => {
            let angles = (640..800).rev();
            draw_cols(angles, grid_origin);
        }
        East => {
            let angles = (0..80).rev().chain((880..960).rev());
            draw_cols(angles, grid_origin);
        }
        West => {
            let angles = (400..560).rev();
            draw_cols(angles, grid_origin);
        }
    }
}

fn find_intersection(grid_origin: (usize, usize), angle_num: usize) -> Intersection {
    // e.g. for North heading will check all angle_nums in range:
    // 90-(FOV/2) -> 90+(FOV/2)
    // angle can be any multiple of FOV/160
    // angle_num between 0 and 959
    let h = find_horizontal_intersection(grid_origin, angle_num);
    let v = find_vertical_intersection(grid_origin, angle_num);

    match (h, v) {
        (None, None) => unreachable!(),
        (Some(hi), None) => hi,
        (None, Some(vi)) => vi,
        (Some(hi), Some(vi)) => {
            if hi.distance <= vi.distance {
                hi
            } else {
                vi
            }
        }
    }
}

fn find_horizontal_intersection(
    grid_origin: (usize, usize),
    angle_num: usize,
) -> Option<Intersection> {
    // Intersections with horizontal grid-lines, y-direction
    // Origin is  middle of block
    let py = (grid_origin.0 * WALL_SIZE) + (WALL_SIZE / 2);
    let px = (grid_origin.1 * WALL_SIZE) + (WALL_SIZE / 2);
    let angle: f32 = ANGLE_DIFF_DEGREES * angle_num as f32;

    let mut ay = if angle > 180.0 {
        grid_origin.0 * WALL_SIZE + WALL_SIZE
    } else {
        grid_origin.0 * WALL_SIZE - 1
    };

    if tan(angle_num) == 0.0 {
        return None;
    }

    let ax_neg = (px as f32 + ((py as i32 - ay as i32) as f32 / tan(angle_num))).floor();
    if ax_neg < 0.0 {
        return None;
    }
    let mut ax: usize = ax_neg as usize;
    let xa: i32 = (WALL_SIZE as f32 / tan(angle_num)).floor() as i32;

    while ay >= 0 && ay < 160 && ax >= 0 && ax < 160 {
        let gridx = ax / WALL_SIZE;
        let gridy = ay / WALL_SIZE;

        unsafe {
            if let Some(c) = BOARD[40 * gridy + gridx] {
                let dist = calculate_distance(px, py, ax, ay, angle_num);
                return Some(Intersection {
                    kind: IntersectionKind::HorizontalGrid,
                    distance: dist,
                    colour: c,
                });
            }
        }

        let ydiff: i32 = if angle > 180.0 {
            WALL_SIZE as i32
        } else {
            -(WALL_SIZE as i32)
        }; // Ya
        ay = (ay as i32 + ydiff) as usize;
        ax = (ax as i32 + xa) as usize;
    }

    None
}

fn find_vertical_intersection(
    grid_origin: (usize, usize),
    angle_num: usize,
) -> Option<Intersection> {
    // Intersections with vertical grid-lines, x-direction
    // Origin is  middle of block
    let py = (grid_origin.0 * WALL_SIZE) + (WALL_SIZE / 2);
    let px = (grid_origin.1 * WALL_SIZE) + (WALL_SIZE / 2);
    let angle: f32 = ANGLE_DIFF_DEGREES * angle_num as f32;

    let mut ax = if angle >= 90.0 && angle <= 270.0 {
        grid_origin.1 * WALL_SIZE - 1
    } else {
        grid_origin.1 * WALL_SIZE + WALL_SIZE
    };

    if angle_num == 240 || angle_num == 720 {
        return None; // divergent tan
    }

    let ay_neg = (py as f32 + ((px as i32 - ax as i32) as f32 * tan(angle_num))).floor();
    if ay_neg < 0.0 {
        return None;
    }

    let mut ay = ay_neg as usize;
    let ya = (WALL_SIZE as f32 * tan(angle_num)).floor() as i32;

    while ax >= 0 && ax < 160 && ay >= 0 && ay < 160 {
        let gridx = ax / WALL_SIZE;
        let gridy = ay / WALL_SIZE;

        unsafe {
            if let Some(c) = BOARD[40 * gridy + gridx] {
                let dist = calculate_distance(px, py, ax, ay, angle_num);
                return Some(Intersection {
                    kind: IntersectionKind::VerticalGrid,
                    distance: dist,
                    colour: c,
                });
            }
        }

        let xdiff: i32 = if angle >= 90.0 && angle <= 270.0 {
            -(WALL_SIZE as i32)
        } else {
            WALL_SIZE as i32
        }; // Ya
        ax = (ax as i32 + xdiff) as usize;
        ay = (ay as i32 + ya) as usize;
    }
    None
}
