use crate::*;

pub const BULLISH: [&str; 11] = [
	"PHEW",
	"YAY",
	"STONKS UP",
	"FART OF\nTHE DEAL",
	"TACOed",
	"LIBERATED",
	"PAUSED LOL",
	"CHICKEN",
	"BOK BOK",
	"CRINGE",
	"LIBS GOT\nOWNED",
];

pub const BEARISH: [&str; 11] = [
	"O NO",
	"LAME",
	"MEIN GOTT",
	"O NEIN",
	"MA CHE SCEMO",
	"CE TAMPIT",
	"STONKS DOWN",
	"WTF",
	"MARKET CRASH\nLOL",
	"RECIPROCATED",
	"MY 401K",
];

pub const TARIFF_VALUES: [&str; 10] = [
	"20",
	"42",
	"69",
	"100",
	"200",
	"420",
	"9001",
	"GAJILLION",
	"BAZMILLION",
	"INFINITY",
];

pub const TARIFF_TARGETS: [&str; 12] = [
	"STEEL",
	"ELECTRONICS",
	"NAZI CARS",
	"THIS GAME",
	"ORANGE TAN",
	"PENGUINS",
	"CHINA",
	"EUROPE",
	"ATLANTIS",
	"BEVY",
	"AI SLOP",
	"ITCH.IO",
];

pub const ENDING_SARCASM: [&str; 9] = [
	"Don't you just love the free market?",
	"So are your friends who knew about it in advance.",
	"And 10x morally poorer.",
	"I love the smell of insider trading in the morning.",
	"I swear it will trickle down to everyone.",
	"Private sector efficiency at its finest.",
	"Trade pertners respect us so much.",
	"much deal very respect so amaze",
	"Hope this doesn't break any bromance.",
];

pub fn random_string(a: &[&'static str]) -> &'static str {
	let mut rng = rand::rng();
	let idx = rng.random_range(..a.len());
	a[idx]
}

pub fn random_tariff() -> String {
	let mut rng = rand::rng();
	let value = TARIFF_VALUES[rng.random_range(..TARIFF_VALUES.len())];
	let target = TARIFF_TARGETS[rng.random_range(..TARIFF_TARGETS.len())];
	format!("{}% TARIFFS\nON {}", value, target)
}

#[derive(Event)]
pub struct OverheadTextRequest {
	pub attached_to: Entity,
	pub text: Option<String>,
	pub duration_sec: Option<f32>,
}

#[derive(Component)]
#[require(Text2d)]
pub struct OverheadText {
	display_timer: Timer,
}

impl Default for OverheadText {
	fn default() -> Self {
		Self {
			display_timer: Timer::from_seconds(0.1, TimerMode::Once),
		}
	}
}

pub fn process_text_requests(
	mut events: EventReader<OverheadTextRequest>,
	parent_q: Query<&Children>,
	mut text_q: Query<(&mut OverheadText, &mut Visibility, &mut Text2d)>,
) {
	for event in events.read() {
		let Ok(children) = parent_q.get(event.attached_to) else {
			warn!("OverheadTextRequest invalid");
			continue;
		};
		for child in children {
			let Ok((mut overhead, mut visibility, mut text)) = text_q.get_mut(*child) else {
				continue;
			};
			*visibility = Visibility::Visible;
			overhead.display_timer =
				Timer::from_seconds(event.duration_sec.unwrap_or(1.), TimerMode::Once);
			if let Some(a) = &event.text {
				text.0 = a.clone()
			}
		}
	}
}

pub fn update_texts(mut q: Query<(&mut OverheadText, &Text2d, &mut Visibility)>, time: Res<Time>) {
	for (mut overhead, _text, mut visibility) in q.iter_mut() {
		if overhead.display_timer.tick(time.delta()).just_finished() {
			*visibility = Visibility::Hidden;
		}
	}
}
