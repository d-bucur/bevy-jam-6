use std::borrow::Cow;

use crate::game_states::GameState;
use crate::*;
use bevy::{color::palettes::basic::*, state::state::FreelyMutableState};

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

pub struct MenuPlugin {}

impl Plugin for MenuPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(OnEnter(GameState::Menu), setup_main_menu)
			.add_systems(OnEnter(GameState::Paused), setup_paused)
			.add_systems(OnEnter(GameState::Options), setup_options)
			.add_systems(OnEnter(GameState::Screensaver), setup_screensaver)
			.add_systems(OnEnter(GameState::Tutorial), setup_tutorial)
			.add_systems(Update, apply_button_styles);
	}
}

// better version here: https://github.com/TheBevyFlock/bevy_new_2d/blob/main/src/menus/main.rs
fn setup_main_menu(mut commands: Commands, assets: Res<AssetServer>) {
	commands
		.spawn((
			make_ui_root("Main Menu"),
			GlobalZIndex(2),
			StateScoped(GameState::Menu),
		))
		.with_children(|parent| {
			parent.spawn((
				Text::new(GAME_NAME),
				TextFont {
					font_size: 100.,
					font: assets.load(FONT_MAIN),
					..default()
				},
				TextColor(Color::Srgba(Srgba::hex("ffc107").unwrap())),
				TextShadow::default(),
			));
			parent
				.spawn(make_button("Play"))
				.observe(change_state(GameState::PlaySetup));
			parent
				.spawn(make_button("Options"))
				.observe(change_state(GameState::Options));
			parent
				.spawn(make_button("Tutorial"))
				.observe(change_state(GameState::Tutorial));
			parent
				.spawn(make_button("Screensaver"))
				.observe(change_state(GameState::Screensaver));
		});
}

pub fn make_ui_root(name: impl Into<Cow<'static, str>>) -> impl Bundle {
	(
		Name::new(name),
		Node {
			position_type: PositionType::Absolute,
			width: Val::Percent(100.0),
			height: Val::Percent(100.0),
			align_items: AlignItems::Center,
			justify_content: JustifyContent::Center,
			flex_direction: FlexDirection::Column,
			padding: UiRect::vertical(Val::Px(50.)),
			row_gap: Val::Px(20.0),
			..default()
		},
		// Don't block picking events for other UI roots.
		Pickable::IGNORE,
	)
}

pub fn change_state<T>(
	new_state: T,
) -> impl Fn(Trigger<'_, Pointer<Click>>, ResMut<'_, NextState<T>>)
where
	T: FreelyMutableState,
{
	move |_: Trigger<Pointer<Click>>, mut next_state: ResMut<NextState<T>>| {
		next_state.set(new_state.clone())
	}
}

pub fn make_button(text: impl Into<String>) -> impl Bundle {
	(
		Node {
			width: Val::Percent(100.0),
			height: Val::Percent(100.0),
			align_items: AlignItems::Center,
			justify_content: JustifyContent::Center,
			..default()
		},
		children![(
			Button,
			Node {
				width: Val::Px(250.0),
				height: Val::Px(65.0),
				border: UiRect::all(Val::Px(3.0)),
				// horizontally center child text
				justify_content: JustifyContent::Center,
				// vertically center child text
				align_items: AlignItems::Center,
				..default()
			},
			BorderColor(Color::BLACK),
			BorderRadius::MAX,
			BackgroundColor(NORMAL_BUTTON),
			children![(
				Text::new(text),
				TextFont {
					// font: asset_server.load("fonts/FiraSans-Bold.ttf"),
					font_size: 33.0,
					..default()
				},
				TextColor(Color::srgb(0.9, 0.9, 0.9)),
				TextShadow::default(),
			)]
		)],
	)
}

fn apply_button_styles(
	mut interaction_query: Query<
		(&Interaction, &mut BackgroundColor, &mut BorderColor),
		(Changed<Interaction>, With<Button>),
	>,
) {
	for (interaction, mut color, mut border_color) in &mut interaction_query {
		match *interaction {
			Interaction::Pressed => {
				*color = Srgba::hex("ffc107").unwrap().into();
				border_color.0 = WHITE.into();
			}
			Interaction::Hovered => {
				*color = HOVERED_BUTTON.into();
				border_color.0 = Color::WHITE;
			}
			Interaction::None => {
				*color = NORMAL_BUTTON.into();
				border_color.0 = Color::BLACK;
			}
		}
	}
}

fn setup_paused(mut commands: Commands) {
	commands
		.spawn((
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
			StateScoped(GameState::Paused),
		))
		.with_children(|parent| {
			parent
				.spawn(make_button("Resume"))
				.observe(change_state(GameState::Playing));
			parent
				.spawn(make_button("Restart"))
				.observe(change_state(GameState::PlaySetup));
			parent
				.spawn(make_button("Menu"))
				.observe(change_state(GameState::Menu));
		});
}

fn setup_options(mut commands: Commands) {
	commands
		.spawn((
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
			StateScoped(GameState::Options),
		))
		.with_children(|parent| {
			parent.spawn(Text::new("Volume"));
			parent
				.spawn(Node {
					flex_direction: FlexDirection::Row,
					..default()
				})
				.with_children(|p| {
					p.spawn(Text::new("Donnie"));
					poor_mans_radio_select(p, AudioType::DonnieVoice);
				});
			parent
				.spawn(Node {
					flex_direction: FlexDirection::Row,
					..default()
				})
				.with_children(|p| {
					p.spawn(Text::new("Music"));
					poor_mans_radio_select(p, AudioType::Music);
				});
			parent
				.spawn(Node {
					flex_direction: FlexDirection::Row,
					..default()
				})
				.with_children(|p| {
					p.spawn(Text::new("Effects"));
					poor_mans_radio_select(p, AudioType::TraderStatusChange);
				});
			parent
				.spawn(make_button("Back"))
				.observe(change_state(GameState::Menu));
		});
}

fn poor_mans_radio_select(
	p: &mut bevy::ecs::relationship::RelatedSpawnerCommands<'_, ChildOf>,
	audio: AudioType,
) {
	p.spawn(make_small_button("0"))
		.observe(update_channel_volume(audio, 0.));
	p.spawn(make_small_button("25"))
		.observe(update_channel_volume(audio, 0.25));
	p.spawn(make_small_button("50"))
		.observe(update_channel_volume(audio, 0.5));
	p.spawn(make_small_button("75"))
		.observe(update_channel_volume(audio, 0.75));
	p.spawn(make_small_button("100"))
		.observe(update_channel_volume(audio, 1.));
}

pub fn make_small_button(text: impl Into<String>) -> impl Bundle {
	(
		Node {
			width: Val::Percent(100.0),
			height: Val::Percent(100.0),
			align_items: AlignItems::Center,
			justify_content: JustifyContent::Center,
			..default()
		},
		children![(
			Button,
			Node {
				width: Val::Px(100.0),
				height: Val::Px(50.0),
				border: UiRect::all(Val::Px(3.0)),
				// horizontally center child text
				justify_content: JustifyContent::Center,
				// vertically center child text
				align_items: AlignItems::Center,
				..default()
			},
			BorderColor(Color::BLACK),
			BorderRadius::MAX,
			BackgroundColor(NORMAL_BUTTON),
			children![(
				Text::new(text),
				TextFont {
					// font: asset_server.load("fonts/FiraSans-Bold.ttf"),
					font_size: 20.0,
					..default()
				},
				TextColor(Color::srgb(0.9, 0.9, 0.9)),
				TextShadow::default(),
			)]
		)],
	)
}

fn setup_screensaver(mut commands: Commands) {
	commands
		.spawn((
			Node {
				position_type: PositionType::Absolute,
				width: Val::Percent(100.0),
				// height: Val::Percent(100.0),
				align_items: AlignItems::Start,
				justify_content: JustifyContent::Start,
				flex_direction: FlexDirection::Row,
				padding: UiRect::vertical(Val::Px(20.)),
				row_gap: Val::Px(20.0),
				..default()
			},
			// Don't block picking events for other UI roots.
			Pickable::IGNORE,
			GlobalZIndex(2),
			// BackgroundColor(bevy::color::palettes::css::BLUE.with_alpha(0.3).into()),
			StateScoped(GameState::Screensaver),
		))
		.with_children(|parent| {
			parent
				.spawn(make_small_button("Back"))
				.observe(change_state(GameState::Menu));
		});
}

fn setup_tutorial(mut commands: Commands) {
	commands
		.spawn((
			Node {
				position_type: PositionType::Absolute,
				width: Val::Percent(100.0),
				height: Val::Percent(100.0),
				align_items: AlignItems::Center,
				justify_content: JustifyContent::Center,
				flex_direction: FlexDirection::Column,
				padding: UiRect::horizontal(Val::Px(200.))
					.with_bottom(Val::Px(50.))
					.with_top(Val::Px(50.)),
				row_gap: Val::Px(20.0),
				..default()
			},
			// Don't block picking events for other UI roots.
			Pickable::IGNORE,
			GlobalZIndex(2),
			StateScoped(GameState::Tutorial),
		))
		.with_children(|parent| {
			parent.spawn((
				Node {
					position_type: PositionType::Relative,
					width: Val::Percent(100.0),
					height: Val::Percent(100.0),
					align_items: AlignItems::Center,
					justify_content: JustifyContent::Center,
					flex_direction: FlexDirection::Column,
					padding: UiRect::all(Val::Px(50.)),
					// margin: UiRect::horizontal(Val::Px(200.)),
					row_gap: Val::Px(20.0),
					flex_grow: 9000.,
					flex_basis: Val::Percent(100.),
					..default()
				},
				BorderRadius::all(Val::Px(20.)),
				BackgroundColor(bevy::color::palettes::css::BLACK.with_alpha(0.8).into()),
				children![(
					Text::new(
						"Donnie launches rumors of tariffs that scare traders and make them BEARISH. This makes stonks go down as they spread in a chain reaction.\nUse the MOUSE to aim TACOs and CLICK to shoot them. TACOs make traders BULLISH again and stonks go up.\nYou have a maximum of 3 TACOs and they slowly recharge.\nThe stonk market alternates between BUY and SELL phases. Press SPACE to buy when stonks are down and SPACE again to sell for a profit.\nMake as much profit as you can in 1 minute rounds!"
					),
					TextFont {
						font_size: 15.0,
						..default()
					},
					TextColor(Color::WHITE),
					// TextShadow::default(),
				)],
			));
			parent
				.spawn(make_button("Menu"))
				.observe(change_state(GameState::Menu));
		});
}
