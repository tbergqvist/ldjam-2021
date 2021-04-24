use macroquad::prelude::*;

use crate::*;

fn get_tile_color(tile_type: &TileType) -> Color {
	match tile_type {
			TileType::Air => BLUE,
			TileType::Ground => BROWN,
			TileType::Gold => YELLOW
	}
}

fn render_tiles(tiles: &Vec<Tile>, camera_offset: f32) {
	let skipped_tiles = camera_offset as usize / TILE_SIZE * WORLD_WIDTH; 
	let tiles_to_render = (WINDOW_HEIGHT as usize / TILE_SIZE + 2) * WORLD_WIDTH;
	for tile in tiles.into_iter()
			.skip(skipped_tiles)
			.take(tiles_to_render) {
			//draw_text(&tile.cell.to_string(), tile.left(), tile.top(), 10., RED);
			draw_rectangle(tile.left(), tile.top() - camera_offset, tile.right() - tile.left(), tile.bottom() - tile.top(), get_tile_color(&tile.tile_type));
	}
}

pub fn render(tiles: &Vec<Tile>, player_state: &game_state::PlayerState) {
	clear_background(BLACK);

	render_tiles(tiles, player_state.camera_offset);

	draw_rectangle(
		player_state.position.left(), 
		player_state.position.top() - player_state.camera_offset, 
		player_state.position.right() - player_state.position.left(), 
		player_state.position.bottom() - player_state.position.top(), 
		GREEN);
draw_text(&get_fps().to_string(), 20., 20., 30., RED);
}