use crate::*;

pub fn setup_audio(asset_server: Res<AssetServer>, mut commands: Commands) {
	// donnie fx
	commands.spawn(AudioPlayer::new(
		asset_server.load("audio/soundboard/Voicy_We have a president who doesn't have a clue.mp3"),
	));
	// other fx
	commands.spawn(AudioPlayer::new(
		asset_server.load("audio/fx/245645__unfa__cartoon-pop-clean.flac"),
	));
	// other fx
	commands.spawn(AudioPlayer::new(
		asset_server.load("audio/fx/9705__lithe-fider__fl_scream-3.wav"),
	));
	
	// soundtrack
	commands.spawn(AudioPlayer::new(
		asset_server.load("audio/music/ytmp3free.cc_taco-man-village-people-trump-edit-youtubemp3free.org.mp3"),
	));
}
