use rand::Rng;

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
	"Me e scemo questo",
	"Ba da asta-i tampit",
	"STONKS DOWN",
	"WTF",
];

pub const tariff_values: [&str; 9] = [
	"20",
	"42",
	"69",
	"100",
	"200",
	"420",
	"9001",
	"GAJILLION",
	"INFINITY",
];

pub const tariff_targets: [&str; 8] = [
	"STEEL",
	"ELECTRONICS",
	"THIS GAME",
	"ORANGE TAN",
	"PENGUINS",
	"CHINA",
	"EUROPE",
	"ATLANTIS",
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