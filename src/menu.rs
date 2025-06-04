use std::borrow::Cow;

use crate::*;
use bevy::color::palettes::basic::*;
use crate::game_states::GameState;

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

pub struct MenuPlugin {}

impl Plugin for MenuPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(OnEnter(GameState::Menu), spawn_main_menu)
			.add_systems(Update, apply_button_styles);
	}
}

// better version here: https://github.com/TheBevyFlock/bevy_new_2d/blob/main/src/menus/main.rs
fn spawn_main_menu(mut commands: Commands) {
	commands
		.spawn((
			make_ui_root("Main Menu"),
			GlobalZIndex(2),
			StateScoped(GameState::Menu),
		))
		.with_children(|parent| {
			parent
				.spawn(make_button("Play"))
				.observe(change_state(GameState::Game));
			parent
				.spawn(make_button("Tutorial"))
				.observe(change_state(GameState::Tutorial));
			parent
				.spawn(make_button("Credits"))
				.observe(change_state(GameState::Credits));
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
			row_gap: Val::Px(20.0),
			..default()
		},
		// Don't block picking events for other UI roots.
		Pickable::IGNORE,
	)
}

pub fn change_state(
	new_state: GameState,
) -> impl Fn(Trigger<'_, Pointer<Click>>, ResMut<'_, NextState<GameState>>) {
	move |_: Trigger<Pointer<Click>>, mut next_state: ResMut<NextState<GameState>>| {
		next_state.set(new_state)
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
				width: Val::Px(200.0),
				height: Val::Px(65.0),
				border: UiRect::all(Val::Px(5.0)),
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
				*color = PRESSED_BUTTON.into();
				border_color.0 = RED.into();
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
