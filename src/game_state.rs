use std::fmt;
use crate::*;

pub struct PlayerState {
	pub position: Position,
	pub camera_offset: f32
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

impl PlayerState {
	pub fn new() -> PlayerState {
			PlayerState{ position: Position { x: 100., y: 0. }, camera_offset: 0. }
	}
}

fn to_tile_cell(x: f32, y: f32) -> usize {
	let cell_x = x as usize / TILE_SIZE;
	let cell_y = y as usize / TILE_SIZE;

	cell_y * WORLD_WIDTH + cell_x
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

pub fn update_player_state(current_state: PlayerState, tiles: &mut Vec<Tile>, input: &PlayerInput) -> PlayerState {
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

	let current_position = &current_state.position;

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

	let new_position = Position{x: current_position.x + vel_x, y: current_position.y + vel_y};
	let camera_offset = f32::max(0., new_position.y - 400.);

	PlayerState {
			position: new_position, 
			camera_offset: camera_offset,
	}
}