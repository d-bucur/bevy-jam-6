use bevy::prelude::*;

use crate::{
	config::{STONKS_DATA_POINTS, STONKS_PER_BEARISH, STONKS_PER_BULLISH, TRADER_COUNT}, StonksTrading, StonksUiText, HEIGHT, WIDTH
};

pub fn ui_update(mut query: Query<&mut Text, With<StonksUiText>>, stonks: Res<StonksTrading>) {
	let mut text = query.single_mut().unwrap();
	**text = format!(
		"Stonks price: {}\nStonks owned: {}\nAverage buy price: {}\nReturns: {}",
		stonks.price_current,
		stonks.owned,
		stonks.avg_buy_price(),
		stonks.returns_total
	);
}

pub fn ui_config_gizmos(mut config_store: ResMut<GizmoConfigStore>) {
	let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
	config.line.width = 5.;
}

pub fn ui_fancy_update(mut gizmos: Gizmos, stonks: Res<StonksTrading>) {
	// TODO make nice chart
	use bevy::color::palettes::css::*;
	const BAR_HEIGHT: f32 = 2.;
	const BAR_WIDTH: f32 = WIDTH / STONKS_DATA_POINTS as f32;
	const BAR_OFFSET: Vec2 = Vec2::new(-WIDTH, -HEIGHT);
	const HUE_MAX: f32 = 123.;
	let min_price: f32 = (TRADER_COUNT * STONKS_PER_BEARISH) as f32;
	let max_price: f32 = (TRADER_COUNT * STONKS_PER_BULLISH) as f32;

	// average buy indicator
	// should use custom style
	gizmos.line_2d(
		BAR_OFFSET + Vec2::new(0., stonks.avg_buy_price() as f32 * BAR_HEIGHT),
		BAR_OFFSET
			+ Vec2::new(
				WIDTH * BAR_WIDTH,
				stonks.avg_buy_price() as f32 * BAR_HEIGHT,
			),
		WHITE,
	);

	// price graph
	gizmos.linestrip_gradient_2d(stonks.price_history.iter().enumerate().map(|(i, &v)| {
		(
			BAR_OFFSET + Vec2::new(i as f32 * BAR_WIDTH, v as f32 * BAR_HEIGHT),
			Hsla::new(
				(v as f32 - min_price) / (max_price - min_price) * HUE_MAX,
				0.7,
				0.5,
				1.,
			),
		)
	}));

	// examples:
	// my_gizmos.arc_2d(Isometry2d::IDENTITY, FRAC_PI_2, 80.0, ORANGE_RED);
	// my_gizmos.long_arc_2d_between(Vec2::ZERO, Vec2::X * 20.0, Vec2::Y * 20.0, ORANGE_RED);
	// my_gizmos.short_arc_2d_between(Vec2::ZERO, Vec2::X * 40.0, Vec2::Y * 40.0, ORANGE_RED);

	// gizmos.linestrip_gradient_2d([
	//     (Vec2::Y * 300., BLUE),
	//     (Vec2::new(-255., -155.), RED),
	//     (Vec2::new(255., -155.), LIME),
	//     (Vec2::Y * 300., BLUE),
	// ]);

	// let domain = Interval::EVERYWHERE;
	// let curve = FunctionCurve::new(domain, |t| Vec2::new(t, ops::sin(t / 25.0) * 100.0));
	// let resolution = 100;
	// let times_and_colors = (0..=resolution)
	//     .map(|n| n as f32 / resolution as f32)
	//     .map(|t| (t - 0.5) * 600.0)
	//     .map(|t| (t, TEAL.mix(&HOT_PINK, (t + 300.0) / 600.0)));
	// gizmos.curve_gradient_2d(curve, times_and_colors);
}
