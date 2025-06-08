use std::collections::VecDeque;

use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy::sprite::Material2d;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use rand::prelude::*;

mod animations;
mod assets;
mod audio;
mod config;
mod dialogue;
mod game_states;
mod menu;
mod movement;
mod physics;
mod shooting;
mod stonks;
mod traders;
mod ui;

use animations::*;
use assets::*;
use audio::*;
use config::*;
use dialogue::*;
use game_states::*;
use menu::*;
use movement::*;
use physics::*;
use shooting::*;
use stonks::*;
use traders::*;
use ui::*;

#[derive(Component)]
struct Donnie;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct PlayerArrowIndicator;

#[derive(Resource)]
struct GameStats {
	total_projectiles_launched: u32,
	time_remaining: Timer,
}

impl Default for GameStats {
	fn default() -> Self {
		Self {
			total_projectiles_launched: 0,
			time_remaining: Timer::from_seconds(ROUND_TIME, TimerMode::Once),
		}
	}
}

fn main() {
	App::new()
		.add_plugins(DefaultPlugins.set(AssetPlugin {
			// Wasm builds will check for meta files (that don't exist) if this isn't set.
			// This causes errors and even panics in web builds on itch.
			// See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
			meta_check: AssetMetaCheck::Never,
			..default()
		}))
		.init_state::<GameState>()
		.insert_resource(StonksTrading::default())
		.insert_resource(AssetsBuffer::default())
		// Enable this part to use inspector
		// .add_plugins(EguiPlugin {
		// 	enable_multipass_for_primary_context: true,
		// })
		// .add_plugins(WorldInspectorPlugin::new())
		.add_plugins(MenuPlugin {})
		.add_plugins(UIIngamePlugin {})
		.add_systems(
			Startup,
			(window_setup, preload_assets, setup_entities, setup_audio).chain(),
		)
		.add_systems(OnEnter(GameState::PlaySetup), setup_play)
		.add_systems(
			// systems that rely on input should be in Update to avoid missing any
			Update,
			(player_shooting, player_investing).run_if(in_state(GameState::Playing)),
		)
		.add_systems(
			// I prefer having most systems in one place to better understand the flow of the game
			FixedUpdate,
			(
				check_game_pause,
				(check_game_over, charge_player_tacos)
					.chain()
					.run_if(in_state(GameState::Playing)),
				(
					donnie_shooting,
					spawn_projectiles,
					process_text_requests,
					update_texts,
					handle_random_movement,
					move_entities,
					animations,
					y_sort,
					check_collisions,
					handle_collisions,
					// debug_colliders,
					tick_trader_timers,
					update_trader_status,
				)
					.chain()
					.run_if(not(in_state(GameState::Paused))),
				(update_stonks_price, ui_update)
					.chain()
					.run_if(in_state(GameState::Playing)),
				(tick_text_effects, ui_update_debug, ui_update_game_stats)
					.chain()
					.run_if(in_state(GameState::Playing)),
				(handle_effect_requests,)
					.chain()
					.run_if(not(in_state(GameState::Paused))),
			)
				.chain(),
		)
		.add_event::<CollisionEvent>()
		.add_event::<TraderChange>()
		.add_event::<SpawnProjectile>()
		.add_event::<OverheadTextRequest>()
		.insert_resource(DonnieShootingLogic::default())
		.insert_resource(GameStats::default())
		.insert_resource(AudioLimitCounters([1, 3, 3, 3]))
		.insert_resource(VolumeSettings::default())
		.insert_resource(ClearColor(Color::Srgba(Srgba::hex("6b6a7b").unwrap())))
		.add_observer(on_stonks_notification)
		.run();
}

fn setup_entities(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<ColorMaterial>>,
) {
	let mut rng = rand::rng();

	// Shadow mesh
	let mesh_handle = meshes.add(Circle::new(25.));
	let material_handle = materials.add(Color::hsva(0., 0., 0.2, 0.5));

	// Traders
	for _ in 0..TRADER_COUNT {
		commands
			.spawn((
				Sprite {
					image: asset_server.load(investor_texture_path()),
					custom_size: Some(vec2(50., 50.)),
					image_mode: SpriteImageMode::Scale(ScalingMode::FitCenter),
					anchor: bevy::sprite::Anchor::BottomCenter,
					..Default::default()
				},
				Transform {
					translation: Vec3::new(
						rng.random_range(-WIDTH..WIDTH),
						rng.random_range(-HEIGHT..HEIGHT),
						0.,
					),
					..Default::default()
				},
				Trader::default(),
				Collider {
					radius: 25.,
					offset: Vec2::new(0., 14.),
				},
				PhysicsBody {
					velocity: get_trader_random_velocity(),
					..Default::default()
				},
				RandomMovement::default(),
				EdgeBehavior::Wraparound,
				wobble_animation(),
				// Shadow
				children![
					shadow(mesh_handle.clone(), material_handle.clone()),
					overhead_text(""),
				],
			))
			.observe(audio::on_trader_status_change);
	}

	// TODO refactor common stuff?
	// Donnie
	commands
		.spawn((
			Name::new("Donnie"),
			Sprite {
				image: asset_server.load(donnie_texture_path()),
				custom_size: Some(vec2(70., 70.)),
				image_mode: SpriteImageMode::Scale(ScalingMode::FitCenter),
				anchor: bevy::sprite::Anchor::BottomCenter,
				..Default::default()
			},
			Transform {
				translation: Vec3::new(0., HEIGHT, 0.),
				..Default::default()
			},
			Collider {
				radius: 25.,
				offset: Vec2::new(0., 14.),
			},
			PhysicsBody {
				velocity: get_trader_random_velocity(),
				..Default::default()
			},
			RandomMovement::default(),
			EdgeBehavior::Wraparound,
			wobble_animation(),
			Donnie,
			// Shadow
			children![
				shadow(mesh_handle.clone(), material_handle.clone()),
				overhead_text("TARIFFS!"),
			],
		))
		.observe(audio::on_donnie_shot);

	// Taco truck
	commands
		.spawn((
			Name::new("Taco Truck"),
			Sprite {
				image: asset_server.load("taco_man3/taco-truck.png"),
				custom_size: Some(vec2(70., 70.)),
				image_mode: SpriteImageMode::Scale(ScalingMode::FitCenter),
				anchor: bevy::sprite::Anchor::BottomCenter,
				..Default::default()
			},
			Transform {
				translation: Vec3::new(0., 0., 0.),
				..Default::default()
			},
			Collider {
				radius: 25.,
				offset: Vec2::new(0., 14.),
			},
			Player,
			PhysicsBody {
				velocity: get_trader_random_velocity(),
				..Default::default()
			},
			RandomMovement::default(),
			EdgeBehavior::Wraparound,
			PlayerShootingLogic::default(),
			wobble_animation(),
			// Shadow
			children![shadow(mesh_handle.clone(), material_handle.clone()),],
		))
		.observe(audio::on_projectile_shot);

	// Player arrow
	commands.spawn((
		Sprite {
			image: asset_server.load("taco_man3/right-arrow.png"),
			custom_size: Some(vec2(50., 50.)),
			image_mode: SpriteImageMode::Scale(ScalingMode::FitCenter),
			anchor: bevy::sprite::Anchor::CenterLeft,
			..Default::default()
		},
		Visibility::Hidden, // TODO temporary hidden. should remove
		// Transform {
		// 	scale: Vec3::new(2., 1., 1.),
		// 	..default()
		// },
		PlayerArrowIndicator,
		SkipYSort,
	));
}

fn setup_play(
	mut cmds: Commands,
	mut next_state: ResMut<NextState<GameState>>,
	q: Query<(Entity), With<Projectile>>,
	mut spawn_events: EventReader<SpawnProjectile>,
) {
	// Reset game stats
	cmds.insert_resource(StonksTrading::default());
	cmds.insert_resource(DonnieShootingLogic::default());
	cmds.insert_resource(GameStats::default());
	for e in q.iter() {
		cmds.entity(e).despawn();
	}
	spawn_events.clear();
	next_state.set(GameState::Playing);
	// Reset trader statuses?
}

fn window_setup(
	mut window: Single<&mut Window>,
	mut cmds: Commands,
	asset_server: Res<AssetServer>,
) {
	// Not really sure why this works, but it's a late fix
	let scale_factor = 2.5;
	window
		.resolution
		.set(WIDTH * scale_factor, HEIGHT * scale_factor);
	cmds.spawn((
		Camera2d,
		Projection::Orthographic(OrthographicProjection {
			// This seems to be the best one?
			scaling_mode: bevy::render::camera::ScalingMode::FixedVertical {
				viewport_height: HEIGHT * scale_factor,
			},
			// scaling_mode: bevy::render::camera::ScalingMode::Fixed {
			// 	width: WIDTH * scale_factor,
			// 	height: HEIGHT * scale_factor,
			// },
			// scaling_mode: bevy::render::camera::ScalingMode::AutoMax {
			// 	max_width: WIDTH * scale_factor,
			// 	max_height: HEIGHT * scale_factor,
			// },
			..OrthographicProjection::default_2d()
		}),
		Transform {
			translation: Vec3::new(0., 50., 0.),
			..default()
		},
	));
	cmds.spawn((
		Name::new("Background"),
		Sprite {
			image: asset_server.load("taco_man3/background.png"),
			custom_size: Some(vec2(WIDTH * scale_factor, HEIGHT * scale_factor)),
			// image_mode: SpriteImageMode::Scale(ScalingMode::FitCenter),
			anchor: bevy::sprite::Anchor::Center,
			..Default::default()
		},
		Transform::from_xyz(0., 0., -900.),
		SkipYSort,
	));
}

fn handle_collisions(
	mut cmds: Commands,
	mut collisions: EventReader<CollisionEvent>,
	mut trader_changes: EventWriter<TraderChange>,
	mut spawn_events: EventWriter<SpawnProjectile>,
	mut trader: Query<&mut Trader>,
	rumor: Query<&Rumor>,
	trader_query: Query<(&Transform, Option<&TraderRestTimer>)>,
	projectile_query: Query<(&Projectile, &Transform)>,
) {
	for collision in collisions.read() {
		// TODO cache component gets?
		let is_rumor_trader =
			rumor.get(collision.entity1).is_ok() && trader.get(collision.entity2).is_ok();
		let is_trader_rumor =
			trader.get(collision.entity1).is_ok() && rumor.get(collision.entity2).is_ok();

		let mut check_rumor_vs_trader = |rumor_entity, trader_entity| {
			let mut trader = trader.get_mut(trader_entity).unwrap();
			let rumor = *rumor.get(rumor_entity).unwrap();
			let (projectile, projectile_transform) = projectile_query.get(rumor_entity).unwrap();
			if projectile.owner == Some(trader_entity)
				|| (rumor == Rumor::Taco && trader.status == TraderStatus::Bullish)
				|| (rumor == Rumor::Tariff && trader.status == TraderStatus::Bearish)
			{
				return false;
			}
			let (trader_transform, maybe_rest) = trader_query.get(trader_entity).unwrap();
			if maybe_rest.is_some() {
				return false;
			}

			// Change trader status
			let new_status = match rumor {
				Rumor::Tariff => TraderStatus::Bearish,
				Rumor::Taco => TraderStatus::Bullish,
			};
			let change_event = TraderChange {
				entity: trader_entity,
				prev: trader.status,
				new: new_status,
			};
			trader_changes.write(change_event.clone());
			trader.status = new_status;
			cmds.entity(trader_entity).insert((
				TraderStatusTimer(Timer::from_seconds(5., TimerMode::Once)),
				TraderRestTimer(Timer::from_seconds(0.5, TimerMode::Once)),
			));

			// Spawn chain reaction bullets
			cmds.entity(rumor_entity).despawn();
			let position = projectile_transform.translation.xy();
			let hit_direction = (trader_transform.translation.xy()
				- projectile_transform.translation.xy())
			.normalize();
			let pattern = UniformPattern { bullet_count: 3 };
			for dir in pattern.direction_iter(hit_direction) {
				spawn_events.write(SpawnProjectile {
					projectile_type: rumor,
					position,
					direction: dir * PROJECTILE_SPEED,
					owner: Some(trader_entity),
				});
			}
			true
		};

		if is_rumor_trader {
			check_rumor_vs_trader(collision.entity1, collision.entity2);
		} else if is_trader_rumor {
			check_rumor_vs_trader(collision.entity2, collision.entity1);
		}
	}
}

fn wobble_animation() -> Animation<Transform> {
	Animation::<Transform> {
		progress: rand::random_range(0.0..=1.0),
		animation_speed: 10.,
		animations: vec![
			AnimValue::new(|t, _, n| t.scale.y = n, |p| (-p * 2.).cos() / 2. * 0.1 + 1.),
			AnimValue::new(|t, o, n| t.rotate_z(-o + n), |p| p.sin() * 0.075),
			AnimValue::new(|t, o, n| t.translation.y += n - o, |p| (-p * 2.).cos() * 5.),
		],
	}
}

fn shadow(mesh: Handle<Mesh>, material: Handle<impl Material2d>) -> impl Bundle {
	(
		Mesh2d(mesh.clone()),
		MeshMaterial2d(material.clone()),
		Transform::from_xyz(0., 0., -2.).with_scale(Vec3::new(1., 0.5, 1.)),
	)
}

fn overhead_text(text: impl Into<String>) -> impl Bundle {
	(
		Text2d::new(text),
		OverheadText::default(),
		TextColor(Color::Srgba(Srgba::hex("ffffff").unwrap())),
		Transform::from_xyz(0., -25., 10.),
		TextLayout::new_with_justify(JustifyText::Center),
		TextFont {
			font_size: 17.,
			..default()
		},
	)
}
