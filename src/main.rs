use macroquad::prelude::*;

use std::fmt;

#[derive(PartialEq)]
enum TileType {
    Air,
    Ground,
    Gold
}

struct Tile {
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

trait Hitbox {
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

struct Position {
    x: f32,
    y: f32
}

impl Hitbox for Position {
    fn top(&self) -> f32 {
        self.y
    }
    fn bottom(&self) -> f32 {
        self.y + 30.0
    }
    fn left(&self) -> f32 {
        self.x
    }
    fn right(&self) -> f32 {
        self.left() + 30.0
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

struct PlayerState {
    position: Position
}

impl PlayerState {
    fn new() -> PlayerState {
        PlayerState{ position: Position { x: 100., y: 0. } }
    }
}

static WORLD_WIDTH: usize = 20;
static WORLD_HEIGHT: usize = 100;
static TILE_SIZE: usize = 40;
static ABOVE_GROUND_ROWS: usize = 5;

fn generate_world() -> Vec<Tile> {

    let mut world_tiles = Vec::<Tile>::new();
    //first five rows are above ground
    let ground_tiles = WORLD_WIDTH * ABOVE_GROUND_ROWS;
    for i in 0..ground_tiles {
        world_tiles.push(Tile::new(i, TileType::Air));
    }

    for i in ground_tiles..WORLD_WIDTH * WORLD_HEIGHT {
        world_tiles.push(Tile::new(i, TileType::Ground));
    }

    world_tiles
}

fn get_tile_color(tile_type: &TileType) -> Color {
    match tile_type {
        TileType::Air => BLUE,
        TileType::Ground => BROWN,
        TileType::Gold => YELLOW
    }
}

fn render_tiles(tiles: &Vec<Tile>) {

    for tile in tiles {
        //draw_text(&tile.cell.to_string(), tile.left(), tile.top(), 10., RED);
        draw_rectangle(tile.left(), tile.top(), tile.right() - tile.left(), tile.bottom() - tile.top(), get_tile_color(&tile.tile_type));
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Yolo".to_owned(),
        window_resizable: false,
        window_width: 800,
        window_height: 600,
        ..Default::default()
    }
}

fn to_tile_cell(x: f32, y: f32) -> usize {
    let cell_x = x as usize / TILE_SIZE;
    let cell_y = y as usize / TILE_SIZE;

    cell_y * WORLD_WIDTH + cell_x
}

fn update_player_state(current_position: Position, tiles: &mut Vec<Tile>, input: &PlayerInput) -> PlayerState {
    let vel_x = if input.left {
        -1.
    } else if input.right {
        1.
    } else {
        0.
    };

    let vel_y = if input.up {
        -3.
    } else {
        //gravity
        3.
    };

    let (tile_cell_y, _, vel_y) = move_prep(0., vel_y, &current_position, tiles);
    let new_pos = Position{x: current_position.x, y: current_position.y + vel_y};
    let (tile_cell_x, vel_x, _) = move_prep(vel_x, 0., &new_pos, tiles);

    let on_ground = if let Some(val) = tile_cell_y {
        new_pos.top() < tiles[val].top()
    } else {
        false
    };

    if tile_cell_y.is_some() && input.down {
        dig(&mut tiles[tile_cell_y.unwrap()]);
    } else if tile_cell_x.is_some() && on_ground {
        dig(&mut tiles[tile_cell_x.unwrap()]);
    }

    PlayerState {
        position: Position{x: current_position.x + vel_x, y: current_position.y + vel_y}
    }
}

fn move_prep(vel_x: f32, vel_y: f32, current_position: &Position, tiles: &Vec<Tile>) -> (Option<usize>, f32, f32) {
    let mut positions_to_scan: Vec<(f32, f32)> = Vec::new(); 
    if vel_y > 0. {
        positions_to_scan.push((current_position.left() + 1., current_position.bottom()));
        positions_to_scan.push((current_position.right() - 1., current_position.bottom()));
    } else if vel_y < 0. {
        positions_to_scan.push((current_position.left() + 1., current_position.top()));
        positions_to_scan.push((current_position.right() - 1., current_position.top()));
    }
    if vel_x > 0. {
        positions_to_scan.push((current_position.right(), current_position.bottom() - 1.));
        positions_to_scan.push((current_position.right(), current_position.top() + 1.));
    } else if vel_x < 0. {
        positions_to_scan.push((current_position.left(), current_position.bottom() - 1.));
        positions_to_scan.push((current_position.left(), current_position.top() + 1.));
    }

    let nearest_tile_idx = positions_to_scan
        .iter()
        .map(|position| {
            to_tile_cell(position.0 + vel_x, position.1 + vel_y)
        })
        .filter(|tile_cell|{
            tiles[*tile_cell].tile_type != TileType::Air
        }).fold(Option::None, |closest, next| {
            closest.map_or(Some(next), |closest|{
                let closest_tile: &Tile = &tiles[closest];
                let next_tile = &tiles[next];
                let dist1 = (closest_tile.left() - current_position.left()).abs() + (closest_tile.top() - current_position.top()).abs();
                let dist2 = (next_tile.left() - current_position.left()).abs() + (next_tile.top() - current_position.top()).abs();
                if dist2 < dist1 {
                    Some(next)
                } else {
                    Some(closest)
                }
            })
        });

        if let Some(val) = nearest_tile_idx {
            let tile = &tiles[val];
            if vel_y > 0. {
                return (nearest_tile_idx, vel_x, tile.top() - current_position.bottom());
            } else if vel_y < 0. {
                return (nearest_tile_idx, vel_x, tile.bottom() - current_position.top());
            }
            if vel_x > 0. {
                return (nearest_tile_idx, tile.left() - current_position.right(), vel_y);
            } else if vel_x < 0. {
                return (nearest_tile_idx, tile.right() - current_position.left(), vel_y);
            }
        };
        
        return (None, vel_x, vel_y);
}

fn dig(tile: &mut Tile) {
    tile.current_hp -= 1;
    if tile.current_hp <= 0 {
        tile.tile_type = TileType::Air;
    }
}

struct PlayerInput {
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
            player = update_player_state(player.position, &mut tiles, &input);
        }

        clear_background(BLACK);
        render_tiles(&tiles);
        draw_rectangle(
            player.position.left(), 
            player.position.top(), 
            player.position.right() - player.position.left(), 
            player.position.bottom() - player.position.top(), 
            GREEN);
        draw_text(&get_fps().to_string(), 20., 20., 30., RED);

        next_frame().await
    }
}