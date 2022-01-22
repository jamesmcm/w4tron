#[cfg(feature = "buddy-alloc")]
mod alloc;
mod cos;
mod raycast;
mod sin;
mod tan;
mod wasm4;
use wasm4::*;

pub fn set_palette(palette: [u32; 4]) {
    unsafe {
        *PALETTE = palette;
    }
}

#[derive(Clone, Copy)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    pub fn left_turn(&self) -> Self {
        use Direction::*;
        match self {
            North => West,
            West => South,
            South => East,
            East => North,
        }
    }
    pub fn right_turn(&self) -> Self {
        use Direction::*;
        match self {
            North => East,
            West => North,
            South => West,
            East => South,
        }
    }
}

enum DrawMode {
    TwoD,
    ThreeD,
}

static mut DRAWMODE: DrawMode = DrawMode::TwoD;
static mut WINNER: Option<u8> = None;
static mut PREV_GAMEPAD: u8 = 0;
static mut FRAME: u8 = 0;
static mut BOARD: [Option<u8>; 1600] = [None; 1600];
static mut PLAYERS: [Player; 2] = [
    Player {
        index: 1,
        direction: Direction::North,
        position: (20, 25),
    },
    Player {
        index: 2,
        direction: Direction::East,
        position: (38, 16),
    },
];

pub fn build_arena() {
    unsafe {
        // Columns
        for r in 0..40 {
            BOARD[r * 40] = Some(0);
            BOARD[r * 40 + 39] = Some(0);
        }
        // Top and bottom row
        for c in 1..39 {
            BOARD[c] = Some(0);
            BOARD[40 * 39 + c] = Some(0);
        }
    }
}

// 4x4 size
// screen 160x160 pixels
// 40x40 units
pub struct Player {
    index: u8,
    direction: Direction,
    position: (usize, usize), // (y,x)
}

pub fn draw_tile(board_pos: (usize, usize), c: u8) {
    let (row, col) = board_pos;
    unsafe {
        for r in 0..4 {
            (*FRAMEBUFFER)[(((row * 4) + r) * 40) + col] = (c << 6) | (c << 4) | (c << 2) | c;
            // (*FRAMEBUFFER)[(((row * 8) + r) * 2 * 20) + (col * 2) + 1] =
            //     (c << 6) | (c << 4) | (c << 2) | c;
        }
    }
}

pub fn draw_board() {
    unsafe {
        for (ix, tile) in BOARD.iter().enumerate() {
            let row = ix / 40;
            let col = ix % 40;
            match tile {
                None => {
                    draw_tile((row, col), 3);
                }
                Some(x) => {
                    draw_tile((row, col), *x);
                }
            }
        }
    }
}

pub fn draw_players() {
    unsafe {
        for p in &PLAYERS {
            draw_tile(p.position, p.index);
        }
    }
}

pub fn input() {
    unsafe {
        let gamepad = *wasm4::GAMEPAD1;
        let just_pressed = gamepad & (gamepad ^ PREV_GAMEPAD);
        if just_pressed & wasm4::BUTTON_LEFT != 0 {
            PLAYERS[0].direction = PLAYERS[0].direction.left_turn();
        } else if just_pressed & wasm4::BUTTON_RIGHT != 0 {
            PLAYERS[0].direction = PLAYERS[0].direction.right_turn();
        } else if just_pressed & wasm4::BUTTON_UP != 0 {
            DRAWMODE = match DRAWMODE {
                DrawMode::TwoD => DrawMode::ThreeD,
                DrawMode::ThreeD => DrawMode::TwoD,
            };
        }

        PREV_GAMEPAD = gamepad;
    }
}

pub fn step() {
    unsafe {
        for p in &mut PLAYERS {
            use Direction::*;
            BOARD[p.position.0 * 40 + p.position.1] = Some(p.index);
            match p.direction {
                North => p.position = (p.position.0 - 1, p.position.1),
                South => p.position = (p.position.0 + 1, p.position.1),
                East => p.position = (p.position.0, p.position.1 + 1),
                West => p.position = (p.position.0, p.position.1 - 1),
            }
            if BOARD[p.position.0 * 40 + p.position.1].is_some() {
                // TODO: Handle draw - i.e. don't move P1 first
                WINNER = if p.index == 1 { Some(2) } else { Some(1) };
            }
            // If player heads collide, AI wins
            if (p.index == 1 && p.position == PLAYERS[1].position)
                || (p.index == 2 && p.position == PLAYERS[0].position)
            {
                WINNER = Some(2);
            }

            // draw_players();
        }
    }
}

pub fn next_ahead(pos: (usize, usize), dir: Direction) -> usize {
    use Direction::*;
    let diff: i32 = match dir {
        North => -40,
        East => 1,
        West => -1,
        South => 40,
    };
    let mut ix = (pos.0 * 40) + pos.1;
    for z in 1..=40 {
        ix = (ix as i32 + diff) as usize;
        unsafe {
            if BOARD[ix as usize].is_some() {
                return z as usize;
            }
        }
    }
    0
}

pub fn next_left(pos: (usize, usize), dir: Direction) -> usize {
    next_ahead(pos, dir.left_turn())
}

pub fn next_right(pos: (usize, usize), dir: Direction) -> usize {
    next_ahead(pos, dir.right_turn())
}

pub fn ai() {
    unsafe {
        let pos = PLAYERS[1].position;
        let dir = PLAYERS[1].direction;
        let na = next_ahead(pos, dir);
        if na > 3 {
            return;
        }
        let nl = next_left(pos, dir);
        let nr = next_right(pos, dir);
        if nl >= nr && nl > na {
            PLAYERS[1].direction = PLAYERS[1].direction.left_turn();
        }
        if nr >= nl && nr > na {
            PLAYERS[1].direction = PLAYERS[1].direction.right_turn();
        }
    }
}

#[no_mangle]
fn start() {
    set_palette([0x686c73, 0x1e88e5, 0xffc107, 0x000000]);
    build_arena();
}

#[no_mangle]
fn update() {
    unsafe {
        if let Some(w) = &WINNER {
            match w {
                1 => {
                    text("You won!", 40, 80);
                }
                2 => {
                    text("You lost!", 40, 80);
                }
                _ => {}
            }
            return;
        };
        match DRAWMODE {
            DrawMode::TwoD => {
                draw_board();
                draw_players();
            }
            DrawMode::ThreeD => {
                raycast::draw_3d(PLAYERS[0].position, PLAYERS[0].direction);
            }
        }
        input();
        ai();
        if FRAME == 0 {
            step();
        }
        FRAME += 1;
        if FRAME > 8 {
            FRAME = 0
        };
    }
}
