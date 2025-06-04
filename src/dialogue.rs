use crate::*;

pub const bullish: [&str; 5] = [
	"PHEW",
	"YAY",
	"STONKS UP",
	"FART OF THE DEAL",
	"TACOed",
];

pub const bearish: [&str; 7] = [
	"O NO",
	"MEIN GOTT",
	"O NEIN",
	"Ma questo e scemo",
	"Ba da asta-i tampit",
	"STONKS DOWN",
	"WTF",
];

pub const tariff_values: [&str; 10] = [
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

pub const tariff_targets: [&str; 9] = [
	"STEEL",
	"ELECTRONICS",
	"THIS GAME",
	"ORANGE TAN",
	"PENGUINS",
	"CHINA",
	"EUROPE",
	"ATLANTIS",
	"BEVY",
];

pub fn random_dialogue(a: &[&'static str]) -> &'static str {
	let mut rng = rand::rng();
	let idx = rng.random_range(..a.len());
	a[idx]
}

pub fn random_tariff() -> String {
	let mut rng = rand::rng();
	let value = tariff_values[rng.random_range(..tariff_values.len())];
	let target = tariff_targets[rng.random_range(..tariff_targets.len())];
	format!("{}% TARIFFS ON {}", value, target)
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
