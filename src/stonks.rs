use crate::*;

#[derive(Resource, Default)]
pub struct StonksTrading {
	pub price_current: u32,
	pub owned: u32,
	pub spent: u32,
	pub returns_total: i64,
	pub price_history: VecDeque<u32>,
}

impl StonksTrading {
	pub fn avg_buy_price(&self) -> u32 {
		if self.owned != 0 {
			self.spent / self.owned
		} else {
			0
		}
	}
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

	let (low, high) = notif_thresholds();
	if price_current <= low && price_prev > low {
		cmds.trigger(StonksPriceNotification::LOW);
	}
	if price_current >= high && price_prev < high {
		cmds.trigger(StonksPriceNotification::HIGH);
	}
}

const fn notif_thresholds() -> (u32, u32) {
	const NOTIF_THRESHOLD: f32 = 0.7;
	const LOWEST: f32 = (STONKS_PER_BEARISH * TRADER_COUNT) as f32;
	const HIGHEST: f32 = (STONKS_PER_BULLISH * TRADER_COUNT) as f32;
	const DIFF: f32 = HIGHEST - LOWEST;
	(
		(DIFF * (1. - NOTIF_THRESHOLD) + LOWEST) as u32,
		(DIFF * NOTIF_THRESHOLD + LOWEST) as u32,
	)
}

pub fn player_investing(key_input: Res<ButtonInput<KeyCode>>, mut stonks: ResMut<StonksTrading>) {
	if key_input.pressed(KeyCode::KeyB) {
		stonks.owned += 1;
		stonks.spent += stonks.price_current;
	}
	if key_input.just_pressed(KeyCode::KeyS) {
		stonks.returns_total += (stonks.owned * stonks.price_current) as i64 - stonks.spent as i64;
		stonks.owned = 0;
		stonks.spent = 0;
	}
}
