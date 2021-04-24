mod renderer;
mod game_state;

use game_state::*;

use macroquad::prelude::*;

#[derive(PartialEq, Clone)]
pub enum TileType {
    Air,
    Ground,
    Gold
}

pub struct Tile {
    cell: usize,
    tile_type: TileType,
    current_hp: i32,
    max_hp: i32
}

impl Tile {
    fn new(cell: usize, tile_type: TileType) -> Tile {
        let max_hp = 10;
        Tile{ cell: cell, tile_type: tile_type, current_hp: max_hp, max_hp: max_hp }
    }
}

pub trait Hitbox {
    fn top(&self) -> f32;
    fn bottom(&self) -> f32;
    fn left(&self) -> f32;
    fn right(&self) -> f32;
}

impl Hitbox for Tile {
    fn top(&self) -> f32 {
        (self.cell / WORLD_WIDTH * TILE_SIZE) as f32
    }
    fn bottom(&self) -> f32 {
        self.top() + TILE_SIZE as f32
    }
    fn left(&self) -> f32 {
        (self.cell % WORLD_WIDTH * TILE_SIZE) as f32
    }
    fn right(&self) -> f32 {
        self.left() + TILE_SIZE as f32
    }
}

static WINDOW_WIDTH: i32 = 800;
static WINDOW_HEIGHT: i32 = 600;
static WORLD_WIDTH: usize = 20;
static WORLD_HEIGHT: usize = 10000;
static TILE_SIZE: usize = 40;
static ABOVE_GROUND_ROWS: usize = 5;


fn generate_world() -> Vec<Tile> {
    let mut world_tiles = Vec::<Tile>::new();
    //first five rows are above ground
    let ground_tiles = WORLD_WIDTH * ABOVE_GROUND_ROWS;
    for i in 0..ground_tiles {
        world_tiles.push(Tile::new(i, TileType::Air));
    }
    if let Ok(time) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        rand::srand(time.as_secs());
    }

    for i in ground_tiles..WORLD_WIDTH * WORLD_HEIGHT {
        let tile_type = if rand::gen_range(0, 100) <= 10 {
            TileType::Gold
        } else {
            TileType::Ground
        };

        world_tiles.push(Tile::new(i, tile_type));
    }

    world_tiles
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Yolo".to_owned(),
        window_resizable: false,
        window_width: WINDOW_WIDTH,
        window_height: WINDOW_HEIGHT,
        ..Default::default()
    }
}

pub struct PlayerInput {
    left: bool,
    right: bool,
    down: bool,
    up: bool,
}

impl PlayerInput {
    fn new() -> PlayerInput {
        PlayerInput{ left: false, right: false, down: false, up: false}
    }

    fn update(&mut self) {
        self.left = is_key_down(KeyCode::A);
        self.right = is_key_down(KeyCode::D);
        self.down = is_key_down(KeyCode::S);
        self.up = is_key_down(KeyCode::W);
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut tiles = generate_world();
    let mut player = PlayerState::new();
    let mut input = PlayerInput::new();

    let mut counter = 0.;
    let frame_time = 1. / 60.;

    loop {
        input.update();

        counter += get_frame_time();
        while counter > frame_time {
            counter -= frame_time;
            player = game_state::update_player_state(player, &mut tiles, &input);
        }
        
        renderer::render(&tiles, &player);
        
        next_frame().await
    }
}