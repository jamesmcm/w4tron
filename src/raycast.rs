use crate::Direction;
use crate::BOARD;

// 160x160 pixels projection plane - 160 columns
// 60 degrees = pi/6 rad FOV
// Angle diff per column (ray) = FOV / 160
// Centre of projection plane = (80,80)
//
// Walls are 4x128x4
// Camera height =65
const FOV: usize = 60; // degrees
const WALL_HEIGHT: usize = 128;
const WALL_SIZE: usize = 4; // x,z
                            // const PROJECTION_DISTANCE: usize = 128; // Approximation of 160 width / tan(pi/6) i.e. 2**7
const SLICE_HEIGHT_CONST: usize = WALL_HEIGHT << 7; // Divide this by distance to get actual height

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

fn find_intersection(grid_origin: (usize, usize), dir: Direction, angle: f32) -> Intersection {
    // e.g. for North heading will check all angles in range:
    // 90-(FOV/2) -> 90+(FOV/2)
    // Angle can be any multiple of FOV/160
    todo!()
}

fn find_horizontal_intersection(grid_origin: (usize, usize), angle: f32) -> Intersection {
    // Intersections with horizontal grid-lines, y-direction
    // Origin is  middle of block
    let py = (grid_origin.0 * WALL_SIZE) + (WALL_SIZE / 2);
    let px = (grid_origin.1 * WALL_SIZE) + (WALL_SIZE / 2);

    let mut ay = if angle > 180.0 {
        grid_origin.0 * WALL_SIZE + WALL_SIZE
    } else {
        grid_origin.0 * WALL_SIZE - 1
    };
    let mut ax = px + (py - ay) / tan(angle);
    let xa = WALL_SIZE / tan(angle);

    while ay > 0 && ay < 160 {
        let gridx = ax / WALL_SIZE;
        let gridy = ay / WALL_SIZE;

        unsafe {
            if let Some(c) = BOARD[gridy + WALL_SIZE * gridx] {
                let mut dist: f32;
                if py != ay {
                    dist = py as f32 - ay as f32;
                    if dist < 0.0 {
                        dist = -1.0 * dist;
                    };
                    dist /= sin(angle);
                } else {
                    dist = px as f32 - ax as f32;
                    if dist < 0.0 {
                        dist = -1.0 * dist;
                    };
                    dist /= cos(angle);
                }
                return Intersection {
                    kind: IntersectionKind::HorizontalGrid,
                    distance: dist,
                    colour: c,
                };
            }
        }

        let ydiff: i32 = if angle > 180.0 {
            WALL_SIZE as i32
        } else {
            -1 * (WALL_SIZE as i32)
        }; // Ya
        ay = (ay as i32 + ydiff) as usize;
        ax += xa;
    }

    unreachable!();
}

fn find_vertical_intersection(grid_origin: (usize, usize), angle: f32) -> Intersection {
    // Intersections with vertical grid-lines, x-direction
    // Origin is  middle of block
    let py = (grid_origin.0 * WALL_SIZE) + (WALL_SIZE / 2);
    let px = (grid_origin.1 * WALL_SIZE) + (WALL_SIZE / 2);

    let mut ax = if angle >= 90.0 && angle <= 270.0 {
        grid_origin.1 * WALL_SIZE + WALL_SIZE
    } else {
        grid_origin.1 * WALL_SIZE - 1
    };
    let mut ay = py + (px - ax) * tan(angle);
    let ya = WALL_SIZE * tan(angle);

    while ax > 0 && ax < 160 {
        let gridx = ax / WALL_SIZE;
        let gridy = ay / WALL_SIZE;

        unsafe {
            if let Some(c) = BOARD[gridy + WALL_SIZE * gridx] {
                let mut dist: f32;
                if py != ay {
                    dist = py as f32 - ay as f32;
                    if dist < 0.0 {
                        dist = -1.0 * dist;
                    };
                    dist /= sin(angle);
                } else {
                    dist = px as f32 - ax as f32;
                    if dist < 0.0 {
                        dist = -1.0 * dist;
                    };
                    dist /= cos(angle);
                }
                return Intersection {
                    kind: IntersectionKind::VerticalGrid,
                    distance: dist,
                    colour: c,
                };
            }
        }

        let xdiff: i32 = if angle >= 90.0 && angle <= 270.0 {
            -1 * (WALL_SIZE as i32)
        } else {
            WALL_SIZE as i32
        }; // Ya
        ax = (ax as i32 + xdiff) as usize;
        ay += ya;
    }

    unreachable!();
}
