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

pub fn update_stonks_price(mut stonks: ResMut<StonksTrading>, query: Query<&Trader>) {
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
	stonks.price_history.push_back(price_current);
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
