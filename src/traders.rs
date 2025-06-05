use crate::*;

#[derive(Default, PartialEq, Clone, Copy)]
pub enum TraderStatus {
	#[default]
	Neutral,
	Bullish,
	Bearish,
}

#[derive(Component, Default)]
pub struct Trader {
	pub status: TraderStatus,
}

/// Changes the trader status after some time
#[derive(Component, Deref, DerefMut)]
#[require(Trader)]
pub struct TraderStatusTimer(pub Timer);

/// Rest time for a trader in which it can't collide with projectiles
/// Avoids chain reactions that flood the game with projectiles
#[derive(Component, Deref, DerefMut)]
#[require(Trader)]
pub struct TraderRestTimer(pub Timer);

#[derive(Event)]
pub struct TraderChange {
	pub entity: Entity,
}

/// Just a graphical update of the sprite and overhead text
pub fn update_trader_status(
	mut traders: Query<(&mut Sprite, &Trader)>,
	mut events: EventReader<TraderChange>,
	mut overhead_events: EventWriter<OverheadTextRequest>,
	asset_server: Res<AssetServer>,
) {
	let mut rng = rand::rng();
	const TEXT_CHANCE: f64 = 0.5;

	for event in events.read() {
		let (mut sprite, trader) = traders.get_mut(event.entity).unwrap();
		match trader.status {
			TraderStatus::Neutral => {
				sprite.image = asset_server.load(investor_texture_path());
			}
			TraderStatus::Bearish => {
				sprite.image = asset_server.load(bearish_texture_path());
				if rng.random_bool(TEXT_CHANCE) {
					overhead_events.write(OverheadTextRequest {
						attached_to: event.entity,
						text: Some(random_dialogue(&BEARISH).to_string()),
						duration_sec: Some(1.),
					});
				}
			}
			TraderStatus::Bullish => {
				sprite.image = asset_server.load(bullish_texture_path());
				if rng.random_bool(TEXT_CHANCE) {
					overhead_events.write(OverheadTextRequest {
						attached_to: event.entity,
						text: Some(random_dialogue(&BULLISH).to_string()),
						duration_sec: Some(1.),
					});
				}
			}
		};
	}
}

pub fn tick_trader_timers(
	time: Res<Time>,
	mut query_status: Query<(&mut TraderStatusTimer, &mut Trader, Entity)>,
	query_rest: Query<(&mut TraderRestTimer, Entity)>,
	mut trader_changes: EventWriter<TraderChange>,
	mut cmds: Commands,
) {
	for (mut timer, mut trader, entity) in &mut query_status {
		if timer.tick(time.delta()).just_finished() {
			trader.status = TraderStatus::Neutral;
			trader_changes.write(TraderChange { entity });
			cmds.entity(entity).remove::<TraderStatusTimer>();
		}
	}

	for (mut timer, entity) in query_rest {
		if timer.tick(time.delta()).just_finished() {
			cmds.entity(entity).remove::<TraderRestTimer>();
		}
	}
}
