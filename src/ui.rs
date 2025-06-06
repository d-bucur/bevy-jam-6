use crate::*;

#[derive(Component)]
pub struct ProfitText;

#[derive(Component)]
pub struct TimeText;

pub struct UIIngamePlugin {}

impl Plugin for UIIngamePlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(
			OnEnter(GameState::Playing),
			(setup_gizmos, setup_game_ui).chain(),
		)
		.init_gizmo_group::<DottedGizmoConfig>();
	}
}

#[derive(GizmoConfigGroup, Default, Reflect)]
pub struct DottedGizmoConfig;

pub fn setup_game_ui(mut commands: Commands) {
	commands
		.spawn((
			Name::new("In game UI"),
			Node {
				position_type: PositionType::Absolute,
				width: Val::Percent(100.0),
				height: Val::Percent(100.0),
				align_items: AlignItems::Start,
				justify_content: JustifyContent::End,
				flex_direction: FlexDirection::Row,
				column_gap: Val::Px(100.0),
				padding: UiRect::vertical(Val::Px(25.)).with_right(Val::Px(200.)),
				..default()
			},
			// Don't block picking events for other UI roots.
			Pickable::IGNORE,
			GlobalZIndex(2),
			StateScoped(GameState::Playing),
		))
		.with_children(|parent| {
			// parent.spawn((Text::new("Stonks go here"), StonksUiText));
			parent.spawn((
				Text::new("Money"),
				ProfitText,
				TextFont {
					font_size: 75.,
					..default()
				},
				TextShadow::default(),
			));
			parent.spawn((
				Text::new("Time"),
				TimeText,
				TextFont {
					font_size: 75.,
					..default()
				},
				TextShadow::default(),
			));
		});
}

pub fn ui_update_debug(
	mut query: Query<&mut Text, With<StonksUiText>>,
	stonks: Res<StonksTrading>,
) {
	// let mut text = query.single_mut().unwrap();
	// **text = format!(
	// 	"Stonks price: {}\nStonks owned: {}\nAverage buy price: {}\nReturns: {}",
	// 	stonks.price_current,
	// 	stonks.owned,
	// 	stonks.avg_buy_price(),
	// 	stonks.returns_total
	// );
}

pub fn setup_gizmos(mut config_store: ResMut<GizmoConfigStore>) {
	let (default, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
	default.line.width = 5.;

	let (dotted, _) = config_store.config_mut::<DottedGizmoConfig>();
	dotted.line.style = GizmoLineStyle::Dashed {
		gap_scale: 5.,
		line_scale: 5.,
	};
}

pub fn ui_update(
	mut gizmos: Gizmos,
	mut gizmos_dotted: Gizmos<DottedGizmoConfig>,
	stonks: Res<StonksTrading>,
) {
	use bevy::color::palettes::css::*;
	const HUE_MAX: f32 = 123.;

	fn price_ratio(v: u32) -> f32 {
		(v as f32 - PRICE_LOWEST) / (PRICE_HIGHEST - PRICE_LOWEST)
	}

	// level border
	gizmos_dotted.rect_2d(
		Isometry2d::IDENTITY,
		Vec2::new(WIDTH * 2., HEIGHT * 2.),
		Color::Srgba(Srgba::hex("85849b").unwrap()),
	);

	// new chart
	let chart_size = Vec2::new(WIDTH / 2., 100.);
	let chart_offset = Vec2::new(-WIDTH, HEIGHT);
	// buy value indicator
	if let Some(buy_price) = stonks.avg_buy_price() {
		let buy_value = chart_offset + Vec2::new(0., buy_price as f32);
		gizmos_dotted.line_2d(
			buy_value,
			buy_value + Vec2::new(chart_size.x, 0.),
			WHITE.with_alpha(0.5),
		);
	}

	// border
	gizmos.rect_2d(
		Isometry2d::from_xy(-WIDTH + chart_size.x / 2., HEIGHT + 70.),
		chart_size,
		Color::Srgba(Srgba::hex("849b85").unwrap()),
	);
	// chart
	let x_step = chart_size.x / STONKS_DATA_POINTS as f32;
	let y_fact = 1.; // should calc properly
	gizmos.linestrip_gradient_2d(stonks.price_history.iter().enumerate().map(|(i, &v)| {
		(
			chart_offset + Vec2::new(i as f32 * x_step, v as f32 * y_fact),
			Hsla::new(price_ratio(v) * HUE_MAX, 0.7, 0.5, 1.),
		)
	}));
}

pub fn ui_update_game_stats(
	mut time_q: Single<&mut Text, With<TimeText>>,
	mut profit_q: Single<&mut Text, (With<ProfitText>, Without<TimeText>)>,
	stonks: Res<StonksTrading>,
	stats: Res<GameStats>,
) {
	time_q.0 = format!("{}", stats.time_remaining.remaining_secs() as u32);
	profit_q.0 = format!("${}", separated_number(stonks.returns_total));
	// let mut text = text_q.single_mut().unwrap();
	// **text = format!(
	// 	"Time: {}\nProfit: {}",
	// 	stats.time_remaining.remaining_secs() as u32,
	// 	stonks.returns_total,
	// );
}

pub fn ui_setup_gameover_screen(mut commands: Commands, stonks: Res<StonksTrading>) {
	print!("Setting up game over screen...");
	commands
		.spawn((
			Name::new("Game over UI"),
			Node {
				position_type: PositionType::Absolute,
				width: Val::Percent(100.0),
				height: Val::Percent(100.0),
				align_items: AlignItems::Center,
				justify_content: JustifyContent::Center,
				flex_direction: FlexDirection::Column,
				padding: UiRect::vertical(Val::Px(200.)),
				row_gap: Val::Px(20.0),
				..default()
			},
			// Don't block picking events for other UI roots.
			Pickable::IGNORE,
			GlobalZIndex(2),
			StateScoped(GameState::GameOver),
		))
		.with_children(|parent| {
			parent.spawn((
				Text::new("Congratulations!\nYou are now richer by"),
				TextLayout {
					justify: JustifyText::Center,
					..default()
				},
			));
			parent.spawn((
				Text::new(format!("${}", separated_number(stonks.returns_total))),
				TextFont {
					font_size: 100.,
					..default()
				},
				TextShadow::default(),
			));
			parent
				.spawn(make_button("Restart"))
				.observe(change_state(GameState::PlaySetup));
			parent
				.spawn(make_button("Main Menu"))
				.observe(change_state(GameState::Menu));
		});
}

fn separated_number(n: i64) -> String {
	// https://stackoverflow.com/a/67834588/3510803
	let mut num = n
		.abs()
		.to_string()
		.as_bytes()
		.rchunks(3)
		.rev()
		.map(str::from_utf8)
		.collect::<Result<Vec<&str>, _>>()
		.unwrap()
		.join("."); // separator
	if n < 0 {
		num = format!("-{num}")
	}
	num
}
