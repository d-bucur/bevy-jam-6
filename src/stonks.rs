use crate::*;

#[derive(Resource, Default)]
pub struct StonksTrading {
	// has more data than strictly needed, for prototyping different ideas
	pub price_current: u32,
	pub owned: u32,
	pub spent: u32,
	pub returns_total: i64,
	pub price_history: VecDeque<u32>,
	pub phase: TradePhase,
}

impl StonksTrading {
	pub fn avg_buy_price(&self) -> Option<u32> {
		if self.owned != 0 {
			Some(self.spent / self.owned)
		} else {
			None
		}
	}
}

#[derive(Default)]
pub enum TradePhase {
	#[default]
	Buy,
	Dump,
}

#[derive(Component)]
pub struct StonksUiText;

#[derive(Event, PartialEq, Eq)]
pub enum StonksPriceNotification {
	HIGH,
	LOW,
}

pub fn update_stonks_price(
	mut stonks: ResMut<StonksTrading>,
	query: Query<&Trader>,
	mut cmds: Commands,
	config: Res<Config>,
) {
	let counts = query
		.iter()
		.map(|t| t.status)
		.fold([0, 0, 0], |mut c, status| {
			c[status as usize] += 1;
			c
		});
	let price_current = STONKS_PER_NEUTRAL * counts[TraderStatus::Neutral as usize]
		+ STONKS_PER_BEARISH * counts[TraderStatus::Bearish as usize]
		+ STONKS_PER_BULLISH * counts[TraderStatus::Bullish as usize];
	stonks.price_current = price_current;

	if stonks.price_history.len() > STONKS_DATA_POINTS as usize {
		stonks.price_history.pop_front();
	}

	let price_prev = *stonks.price_history.back().unwrap_or(&0);
	stonks.price_history.push_back(price_current);

	let (low, high) = notif_thresholds(config);
	if price_current <= low && price_prev > low {
		cmds.trigger(StonksPriceNotification::LOW);
	}
	if price_current >= high && price_prev < high {
		cmds.trigger(StonksPriceNotification::HIGH);
	}
}

fn notif_thresholds(config: Res<Config>) -> (u32, u32) {
	const NOTIF_THRESHOLD: f32 = 0.7;
	let price_lowest = config.price_lowest();
	let diff: f32 = config.price_highest() - price_lowest;
	(
		(diff * (1. - NOTIF_THRESHOLD) + price_lowest) as u32,
		(diff * NOTIF_THRESHOLD + price_lowest) as u32,
	)
}

pub fn player_investing(
	key_input: Res<ButtonInput<KeyCode>>,
	touch_res: Res<Touches>,
	mut stonks: ResMut<StonksTrading>,
	mut effects: EventWriter<TextEffectRequest>,
) {
	let touch_buy = (touch_res.any_just_released() && touch_res.iter().count() == 1) // one released in this frame, one remaining
		|| touch_res.iter_just_released().count() == 2; // both release in the same frame
	if !key_input.just_pressed(KeyCode::Space) && !touch_buy {
		return;
	}
	match stonks.phase {
		TradePhase::Buy => {
			stonks.owned += STONKS_PER_BUY_ACTION;
			stonks.spent += stonks.price_current * STONKS_PER_BUY_ACTION;
			stonks.phase = TradePhase::Dump;

			effects.write(TextEffectRequest {
				text: "BOUGHT".into(),
				duration_sec: 1.,
			});
		}
		TradePhase::Dump => {
			let profit = (stonks.owned * stonks.price_current) as i64 - stonks.spent as i64;
			stonks.returns_total += profit;
			stonks.owned = 0;
			stonks.spent = 0;
			stonks.phase = TradePhase::Buy;

			effects.write(TextEffectRequest {
				text: format_money(profit),
				duration_sec: 1.,
			});
		}
	}
	// old code where you can buy multiple stonks
	// if key_input.pressed(KeyCode::KeyB) {
	// 	stonks.owned += 1;
	// 	stonks.spent += stonks.price_current;
	// }
	// if key_input.just_pressed(KeyCode::KeyS) {
	// 	stonks.returns_total += (stonks.owned * stonks.price_current) as i64 - stonks.spent as i64;
	// 	stonks.owned = 0;
	// 	stonks.spent = 0;
	// }
}
