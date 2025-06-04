use crate::*;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
#[states(scoped_entities)]
pub enum GameState {
	Menu,
	#[default]
	Game,
	Tutorial,
	Credits,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
#[states(scoped_entities)]
pub enum InGameState {
	#[default]
	Playing,
	Paused,
	GameOver,
}

pub fn check_game_over(
	mut stats: ResMut<GameStats>,
	time: Res<Time>,
	mut next_state: ResMut<NextState<InGameState>>,
) {
	if stats.time_remaining.tick(time.delta()).just_finished() {
		next_state.set(InGameState::GameOver);
	}
}

// TODO add pause game