use std::fmt;
use crate::*;

pub struct PlayerState {
	pub position: Position,
	pub camera_offset: f32,
	pub money: u32,
	pub dig_cooldown_s: f64,
	pub next_dig_time: f64
}

impl PlayerState {
	pub fn new() -> PlayerState {
			PlayerState{ 
				position: Position { x: 100., y: 0. }, 
				camera_offset: 0., 
				money: 0,
				dig_cooldown_s: 1.,
				next_dig_time: 0.
			}
	}
}

pub struct Position {
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

fn to_tile_cell(x: f32, y: f32) -> usize {
	let cell_x = x as usize / TILE_SIZE;
	let cell_y = y as usize / TILE_SIZE;

	cell_y * WORLD_WIDTH + cell_x
}

fn find_nearest_tile(vel_x: f32, vel_y: f32, current_position: &Position, tiles: &Vec<Tile>) -> Option<usize> {
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

	positions_to_scan
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
		})
}

fn move_prep(vel_x: f32, vel_y: f32, current_position: &Position, tiles: &Vec<Tile>) -> (f32, f32) {
	find_nearest_tile(vel_x, vel_y, current_position, tiles)
		.map(|nearest_tile| {
			let tile = &tiles[nearest_tile];
			if vel_y > 0. {
					return (vel_x, tile.top() - current_position.bottom());
			} else if vel_y < 0. {
					return (vel_x, tile.bottom() - current_position.top());
			}
			if vel_x > 0. {
					return (tile.left() - current_position.right(), vel_y);
			} else if vel_x < 0. {
					return (tile.right() - current_position.left(), vel_y);
			}
			return (vel_x, vel_y);
		}).unwrap_or((vel_x, vel_y))
}

fn dig(tile: &mut Tile) -> Option<TileType> {
	tile.current_hp -= 1;
	println!("dig");
	if tile.current_hp <= 0 {
			let response = Some(tile.tile_type.clone());
			tile.tile_type = TileType::Air;
			response
	} else {
		None
	}
}

fn get_new_position(current_position: &Position, tiles: &mut Vec<Tile>, input: &PlayerInput) -> Position {
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

	let (_, vel_y) = move_prep(0., vel_y, &current_position, tiles);
	let new_pos = Position{x: current_position.x, y: current_position.y + vel_y};
	let (vel_x, _) = move_prep(vel_x, 0., &new_pos, tiles);
	Position{x: current_position.x + vel_x, y: current_position.y + vel_y}
}

fn get_new_dig_data(current_state: &PlayerState, next_position: &Position, tiles: &mut Vec<Tile>, input: &PlayerInput) -> (f64, u32){
	let nearest_dig_tile = find_nearest_tile(0., 1., &next_position, tiles)
		.and_then(|tile_below| {
			if input.left {
				find_nearest_tile(-1., 0., &next_position, tiles)
			} else if input.right {
				find_nearest_tile(1., 0., &next_position, tiles)
			} else if input.down {
				Some(tile_below)
			} else {
				None
			}
		});

	let game_time = get_time();
	nearest_dig_tile
		.filter(|_| game_time >= current_state.next_dig_time )
		.map(|tile_cell| {
			(
				game_time + current_state.dig_cooldown_s,
				dig(&mut tiles[tile_cell])
			)
		})
		.map(|tile_digged| {
			match tile_digged.1 {
				Some(TileType::Gold) => (tile_digged.0, 5),
				_ => (tile_digged.0, 0)
			}
		}).unwrap_or((current_state.next_dig_time, 0))
}

pub fn update_player_state(current_state: PlayerState, tiles: &mut Vec<Tile>, input: &PlayerInput) -> PlayerState {
	let next_position = get_new_position(&current_state.position, tiles, input);
	let (next_dig_time, money) = get_new_dig_data(&current_state, &next_position, tiles, input);
	let camera_offset = f32::max(0., next_position.y - 400.);
	PlayerState {
			position: next_position, 
			camera_offset: camera_offset,
			money: current_state.money + money,
			next_dig_time: next_dig_time,
			..current_state
	}
}