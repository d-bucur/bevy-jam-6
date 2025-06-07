use crate::*;

#[derive(Component)]
pub struct ProfitText;

#[derive(Component)]
pub struct TimeText;

#[derive(Component)]
pub struct StonkPhaseText;

pub struct UIIngamePlugin {}

impl Plugin for UIIngamePlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(
			OnEnter(GameState::Playing),
			(setup_gizmos, setup_game_ui).chain(),
		)
		.add_systems(
			Update,
			ui_update_stonks_phase.run_if(resource_changed::<StonksTrading>),
		)
		.init_gizmo_group::<DottedGizmoConfig>()
		.add_event::<TextEffectRequest>();
	}
}

const CHART_SIZE: Vec2 = Vec2::new(WIDTH / 2., 100.);
const CHART_OFFSET: Vec2 = Vec2::new(-WIDTH, HEIGHT);

#[derive(GizmoConfigGroup, Default, Reflect)]
pub struct DottedGizmoConfig;

#[derive(Event)]
pub struct TextEffectRequest {
	pub text: String,
	pub duration_sec: f32,
}

pub fn setup_game_ui(mut commands: Commands, window: Single<&Window>) {
	// Bevy UI is PAIN mode
	commands
		.spawn((
			Name::new("In game UI - stats"),
			Node {
				position_type: PositionType::Absolute,
				width: Val::VMin(70.),
				height: Val::Percent(15.),
				top: Val::Px(0.),
				left: Val::Percent(50.),
				column_gap: Val::Px(100.0),
				align_items: AlignItems::Center,
				justify_content: JustifyContent::End,
				padding: UiRect::right(Val::Px(50.)),
				..default()
			},
			// BackgroundColor(bevy::color::palettes::css::BLUE.with_alpha(0.3).into()),
			// Don't block picking events for other UI roots.
			Pickable::IGNORE,
			GlobalZIndex(2),
			StateScoped(GameState::Playing),
		))
		.with_children(|parent| {
			const TEXT_SIZE: f32 = 60.;
			// Indicator displaying profit when selling
			// parent.spawn((
			// 	Text::new("Added"),
			// 	TextFont {
			// 		font_size: TEXT_SIZE,
			// 		..default()
			// 	},
			// 	TextShadow::default(),
			// 	Visibility::Hidden,
			// ));
			parent.spawn((
				Text::new("Money"),
				ProfitText,
				TextFont {
					font_size: TEXT_SIZE,
					..default()
				},
				TextShadow::default(),
			));
			parent.spawn((
				Text::new("Time"),
				TimeText,
				TextFont {
					font_size: TEXT_SIZE,
					..default()
				},
				TextShadow::default(),
			));
		});

	// Text indicator on stonks
	commands
		.spawn((
			Name::new("In game UI - stonks"),
			Node {
				position_type: PositionType::Absolute,
				width: Val::VMin(70.),
				height: Val::Percent(15.),
				top: Val::Px(0.),
				right: Val::Percent(50.),
				align_items: AlignItems::Center,
				justify_content: JustifyContent::Center,
				..default()
			},
			// BackgroundColor(bevy::color::palettes::css::RED.with_alpha(0.3).into()),
			// Don't block picking events for other UI roots.
			Pickable::IGNORE,
			GlobalZIndex(2),
			StateScoped(GameState::Playing),
		))
		.with_children(|parent| {
			parent
				.spawn((
					Node {
						position_type: PositionType::Relative,
						width: Val::VMin(30.),
						height: Val::Percent(100.),
						top: Val::VMin(5.),
						right: Val::VMin(15.),
						align_items: AlignItems::Center,
						justify_content: JustifyContent::Center,
						..default()
					},
					// BackgroundColor(bevy::color::palettes::css::GREEN.with_alpha(0.5).into()),
				))
				.with_children(|parent| {
					parent.spawn((
						Text::new("Sell"),
						TextFont {
							font_size: 15.,
							..default()
						},
						StonkPhaseText,
					));
				});
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
	// buy value indicator
	if let Some(buy_price) = stonks.avg_buy_price() {
		let buy_value = CHART_OFFSET + Vec2::new(0., buy_price as f32);
		gizmos_dotted.line_2d(
			buy_value,
			buy_value + Vec2::new(CHART_SIZE.x, 0.),
			WHITE.with_alpha(0.5),
		);
	}

	// border
	gizmos.rect_2d(
		Isometry2d::from_xy(-WIDTH + CHART_SIZE.x / 2., HEIGHT + 70.),
		CHART_SIZE,
		Color::Srgba(Srgba::hex("849b85").unwrap()),
	);
	// chart
	let x_step = CHART_SIZE.x / STONKS_DATA_POINTS as f32;
	let y_fact = 1.; // should calc properly
	gizmos.linestrip_gradient_2d(stonks.price_history.iter().enumerate().map(|(i, &v)| {
		(
			CHART_OFFSET + Vec2::new(i as f32 * x_step, v as f32 * y_fact),
			Hsla::new(price_ratio(v) * HUE_MAX, 0.7, 0.5, 1.),
		)
	}));
}

pub fn handle_effect_requests(mut effects: EventReader<TextEffectRequest>) {
	for e in effects.read() {
		// todo!();
	}
}

pub fn ui_update_stonks_phase(
	stonks: Res<StonksTrading>,
	mut text: Single<&mut Text, With<StonkPhaseText>>,
) {
	text.0 = (match stonks.phase {
		TradePhase::Buy => "Buy",
		TradePhase::Dump => "Sell",
	})
	.to_string()
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
