use crate::*;

#[derive(Default, PartialEq, Clone, Copy, Debug)]
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

#[derive(Event, Clone)]
pub struct TraderChange {
	pub entity: Entity,
	pub prev: TraderStatus,
	pub new: TraderStatus,
}

/// Just a graphical update of the sprite and overhead text
pub fn update_trader_status(
	mut traders: Query<(&mut Sprite, &Trader, Entity)>,
	mut events: EventReader<TraderChange>,
	mut overhead_events: EventWriter<OverheadTextRequest>,
	asset_server: Res<AssetServer>,
	mut cmds: Commands,
) {
	let mut rng = rand::rng();
	const TEXT_CHANCE: f64 = 0.5;

	for event in events.read() {
		let (mut sprite, trader, entity) = traders.get_mut(event.entity).unwrap();
		match trader.status {
			TraderStatus::Neutral => {
				sprite.image = asset_server.load(investor_texture_path());
			}
			TraderStatus::Bearish => {
				sprite.image = asset_server.load(bearish_texture_path());
				cmds.trigger_targets(event.clone(), entity);
				if rng.random_bool(TEXT_CHANCE) {
					overhead_events.write(OverheadTextRequest {
						attached_to: event.entity,
						text: Some(random_string(&BEARISH).to_string()),
						duration_sec: Some(1.2),
					});
				}
			}
			TraderStatus::Bullish => {
				sprite.image = asset_server.load(bullish_texture_path());
				cmds.trigger_targets(event.clone(), entity);
				if rng.random_bool(TEXT_CHANCE) {
					overhead_events.write(OverheadTextRequest {
						attached_to: event.entity,
						text: Some(random_string(&BULLISH).to_string()),
						duration_sec: Some(1.2),
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
			let change_event = TraderChange {
				entity,
				prev: trader.status,
				new: TraderStatus::Neutral,
			};
			trader.status = TraderStatus::Neutral;
			trader_changes.write(change_event.clone());
			cmds.trigger_targets(change_event, entity);
			cmds.entity(entity).remove::<TraderStatusTimer>();
		}
	}

	for (mut timer, entity) in query_rest {
		if timer.tick(time.delta()).just_finished() {
			cmds.entity(entity).remove::<TraderRestTimer>();
		}
	}
}
