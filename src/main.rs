use std::collections::VecDeque;

use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use rand::prelude::*;

mod animations;
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

#[derive(Resource)]
struct GameStats {
	total_projectiles_launched: u32,
	time_remaining: Timer,
	tacos_remaining: u32,
}

impl Default for GameStats {
	fn default() -> Self {
		Self {
			tacos_remaining: 10,
			total_projectiles_launched: 0,
			time_remaining: Timer::from_seconds(5., TimerMode::Once),
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
		.init_state::<InGameState>()
		// .add_plugins(EguiPlugin {
		// 	enable_multipass_for_primary_context: true,
		// })
		// .add_plugins(WorldInspectorPlugin::new())
		.add_systems(Startup, window_setup)
		.add_plugins(MenuPlugin {})
		.add_systems(
			OnEnter(GameState::Game),
			(setup_entities, ui_config_gizmos, setup_game_ui).chain(),
		)
		.add_systems(
			OnEnter(InGameState::GameOver),
			(ui_setup_gameover_screen).chain(),
		)
		.add_systems(
			FixedUpdate,
			(
				check_game_over,
				player_shooting,
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
				update_stonks_price,
				player_investing,
				ui_update,
				ui_update_game_stats,
				ui_fancy_update,
			)
				.chain()
				.run_if(in_state(GameState::Game))
				.run_if(in_state(InGameState::Playing)),
		)
		.add_event::<CollisionEvent>()
		.add_event::<TraderChange>()
		.add_event::<SpawnProjectile>()
		.add_event::<OverheadTextRequest>()
		.init_resource::<StonksTrading>()
		.init_resource::<DonnieShootingLogic>()
		.insert_resource(ClearColor(Color::Srgba(Srgba::hex("5E5E5E").unwrap())))
		.init_resource::<GameStats>()
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
		commands.spawn((
			Sprite {
				image: asset_server.load("ducky.png"),
				custom_size: Some(vec2(50., 50.)),
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
			Animation::<Transform> {
				progress: 0.,
				animation_speed: 10.,
				animations: vec![
					AnimValue::new(|t, _, n| t.scale.y = n, |p| (-p * 2.).cos() / 2. * 0.1 + 1.),
					AnimValue::new(|t, o, n| t.rotate_z(-o + n), |p| p.sin() * 0.075),
					AnimValue::new(|t, o, n| t.translation.y += n - o, |p| (-p * 2.).cos() * 5.),
				],
			},
			// Shadow
			children![
				(
					Mesh2d(mesh_handle.clone()),
					MeshMaterial2d(material_handle.clone()),
					Transform::from_xyz(0., 0., -2.).with_scale(Vec3::new(1., 0.5, 1.)),
				),
				(Text2d::new("Quack!"), OverheadText::default()),
			],
		));
	}

	// TODO should refactor with above
	// Donnie
	commands.spawn((
		Sprite {
			image: asset_server.load("monkey-svgrepo-com.png"),
			custom_size: Some(vec2(50., 50.)),
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
		Animation::<Transform> {
			progress: 0.,
			animation_speed: 10.,
			animations: vec![
				AnimValue::new(|t, _, n| t.scale.y = n, |p| (-p * 2.).cos() / 2. * 0.1 + 1.),
				AnimValue::new(|t, o, n| t.rotate_z(-o + n), |p| p.sin() * 0.075),
				AnimValue::new(|t, o, n| t.translation.y += n - o, |p| (-p * 2.).cos() * 5.),
			],
		},
		Donnie,
		// Shadow
		children![
			(
				Mesh2d(mesh_handle.clone()),
				MeshMaterial2d(material_handle.clone()),
				Transform::from_xyz(0., 0., -2.).with_scale(Vec3::new(1., 0.5, 1.)),
			),
			(Text2d::new("TARIFFS!"), OverheadText::default())
		],
	));
}

fn window_setup(mut window: Single<&mut Window>, mut cmds: Commands) {
	let scale_factor = window.resolution.scale_factor();
	window
		.resolution
		.set(WIDTH * scale_factor, HEIGHT * scale_factor);
	cmds.spawn((
		Camera2d,
		Projection::Orthographic(OrthographicProjection {
			// scaling_mode: bevy::render::camera::ScalingMode::Fixed {
			// 	width: WIDTH * scale_factor,
			// 	height: HEIGHT * scale_factor,
			// },
			// scaling_mode: bevy::render::camera::ScalingMode::AutoMax {
			// 	max_width: WIDTH * scale_factor,
			// 	max_height: HEIGHT * scale_factor,
			// },
			// This seems to be the best one?
			scaling_mode: bevy::render::camera::ScalingMode::FixedVertical {
				viewport_height: HEIGHT * scale_factor,
			},
			..OrthographicProjection::default_2d()
		}),
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
			trader.status = match rumor {
				Rumor::Tariff => TraderStatus::Bearish,
				Rumor::Taco => TraderStatus::Bullish,
			};
			trader_changes.write(TraderChange {
				entity: trader_entity,
			});
			cmds.entity(trader_entity).insert((
				TraderStatusTimer(Timer::from_seconds(5., TimerMode::Once)),
				TraderRestTimer(Timer::from_seconds(0.5, TimerMode::Once)),
			));

			// Spawn chain reaction bullets
			cmds.entity(rumor_entity).despawn();
			let position = trader_transform.translation.xy();
			let hit_direction = -(position - projectile_transform.translation.xy()).normalize();
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
