use crate::*;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
#[states(scoped_entities)]
pub enum GameState {
	#[default]
	Menu,
	PlaySetup,
	Playing,
	Paused,
	GameOver,
	Tutorial,
	Options,
}

pub fn check_game_over(
	mut stats: ResMut<GameStats>,
	time: Res<Time>,
	mut next_state: ResMut<NextState<GameState>>,
) {
	if stats.time_remaining.tick(time.delta()).just_finished() {
		next_state.set(GameState::GameOver);
	}
}

pub fn check_game_pause(
	key_button: Res<ButtonInput<KeyCode>>,
	mut next_state: ResMut<NextState<GameState>>,
	current_state_res: Res<State<GameState>>,
) {
	if !key_button.just_pressed(KeyCode::Escape) {
		return;
	}
	let current_state = *current_state_res.get();
	if current_state == GameState::Paused {
		next_state.set(GameState::Playing);
	}
	else if current_state == GameState::Playing {
		next_state.set(GameState::Paused);
	}
}