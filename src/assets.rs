use crate::*;

pub fn donnie_texture_path() -> String {
	format!("taco_man3/donnie{}.PNG", rand::random_range(1..=6))
}

pub fn investor_texture_path() -> String {
	format!("taco_man3/investor{}.PNG", rand::random_range(1..=2))
}

pub fn bullish_texture_path() -> String {
	format!("taco_man3/bullish{}.PNG", rand::random_range(1..=3))
}

pub fn bearish_texture_path() -> String {
	format!("taco_man3/bearish{}.PNG", rand::random_range(1..=2))
}

pub fn preload_assets(asset_server: Res<AssetServer>,) {
	asset_server.load_folder("/");
}