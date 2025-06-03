use std::collections::VecDeque;

use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use rand::prelude::*;

mod animations;
mod config;
mod physics;
mod ui;

use animations::*;
use config::*;
use physics::*;
use ui::*;

#[derive(Default, PartialEq, Clone, Copy)]
enum TraderStatus {
	#[default]
	Neutral,
	Bullish,
	Bearish,
}

#[derive(Component, Default)]
struct Trader {
	status: TraderStatus,
}

/// Changes the trader status after some time
#[derive(Component, Deref, DerefMut)]
#[require(Trader)]
struct TraderStatusTimer(Timer);

/// Rest time for a trader in which it can't collide with projectiles
/// Avoids chain reactions that flood the game with projectiles
#[derive(Component, Deref, DerefMut)]
#[require(Trader)]
struct TraderRestTimer(Timer);

#[derive(Component, PartialEq, Clone, Copy)]
enum Rumor {
	Tariff,
	Taco,
}

#[derive(Component)]
enum EdgeBehavior {
	Wraparound,
	Destroy,
}

#[derive(Component)]
struct RandomMovement;

#[derive(Component, Default)]
struct WalkAnimation {
	// TODO progress without overflow
	progress: f32,
}

#[derive(Component)]
struct Projectile {
	owner: Option<Entity>,
}

#[derive(Component)]
struct StonksUiText;

#[derive(Resource, Default)]
struct StonksTrading {
	price_current: u32,
	owned: u32,
	spent: u32,
	returns_total: i64,
	price_history: VecDeque<u32>,
}

impl StonksTrading {
	fn avg_buy_price(&self) -> u32 {
		if self.owned != 0 {
			self.spent / self.owned
		} else {
			0
		}
	}
}

#[derive(Resource)]
struct DonnieShootingLogic {
	shooting_timer: Timer,
}

impl Default for DonnieShootingLogic {
	fn default() -> Self {
		Self {
			shooting_timer: Timer::from_seconds(3.0, TimerMode::Repeating),
		}
	}
}

#[derive(Event)]
struct TraderChange {
	entity: Entity,
}

#[derive(Event)]
struct SpawnProjectile {
	projectile_type: Rumor,
	position: Vec2,
	direction: Vec2,
	owner: Option<Entity>, // TODO replace with relationship
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
		// .add_plugins(EguiPlugin {
		// 	enable_multipass_for_primary_context: true,
		// })
		// .add_plugins(WorldInspectorPlugin::new())
		.add_systems(
			Startup,
			(setup_entities, ui_config_gizmos, window_setup).chain(),
		)
		.add_systems(
			FixedUpdate,
			(
				player_shooting,
				donnie_shooting,
				spawn_projectiles,
				move_entities,
				animations,
				y_sort,
				check_collisions,
				handle_collisions,
				tick_trader_timers,
				update_trader_status,
				update_stonks_price,
				player_investing,
				ui_update,
				ui_fancy_update,
			)
				.chain(),
		)
		.add_event::<CollisionEvent>()
		.add_event::<TraderChange>()
		.add_event::<SpawnProjectile>()
		.init_resource::<StonksTrading>()
		.init_resource::<DonnieShootingLogic>()
		.insert_resource(ClearColor(Color::Srgba(Srgba::hex("5E5E5E").unwrap())))
		.run();
}

fn setup_entities(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<ColorMaterial>>,
) {
	commands.spawn(Camera2d);
	let mut rng = rand::rng();
	// Shadow mesh
	let mesh_handle = meshes.add(Circle::new(25.));
	let material_handle = materials.add(Color::hsva(0., 0., 0.2, 0.5));

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
			Collider { radius: 25. },
			PhysicsBody {
				velocity: Vec2::new(
					rng.random_range(-TRADER_MAX_VELOCITY..TRADER_MAX_VELOCITY),
					rng.random_range(-TRADER_MAX_VELOCITY..TRADER_MAX_VELOCITY),
				),
				..Default::default()
			},
			RandomMovement,
			EdgeBehavior::Wraparound,
			WalkAnimation::default(),
			Animation::<Transform> {
				progress: 0.,
				animation_speed: 10.,
				animations: vec![
					// TODO better to change custom anchor on sprite than transform
					AnimValue::new(|t, _, n| t.scale.y = n, |p| (-p * 2.).cos() / 2. * 0.1 + 1.),
					AnimValue::new(|t, o, n| t.rotate_z(-o + n), |p| p.sin() * 0.075),
					AnimValue::new(|t, o, n| t.translation.y += n - o, |p| (-p * 2.).cos() * 5.),
				],
			},
			// Shadow
			children![(
				Mesh2d(mesh_handle.clone()),
				MeshMaterial2d(material_handle.clone()),
				Transform::from_xyz(0., 0., -2.).with_scale(Vec3::new(1., 0.5, 1.)),
			)],
		));
	}

	commands.spawn((Text::new("Stonks go here"), StonksUiText));
}

fn window_setup(mut window: Single<&mut Window>) {
	let scale_factor = window.resolution.scale_factor();
	window
		.resolution
		.set(WIDTH * scale_factor, HEIGHT * scale_factor);
}

fn move_entities(
	mut query: Query<(
		Entity,
		&mut PhysicsBody,
		&mut Transform,
		Option<&EdgeBehavior>,
		Option<&mut Sprite>,
	)>,
	mut cmds: Commands,
) {
	for (entity, body, mut transform, maybe_edge, mut maybe_sprite) in query.iter_mut() {
		// transform.translation = ((transform.translation.xy() + body.velocity), 0.).into()
		transform.translation.x += body.velocity.x;
		transform.translation.y += body.velocity.y;
		if let Some(mut s) = maybe_sprite {
			s.flip_x = body.velocity.x < 0.
		}

		match maybe_edge {
			Some(EdgeBehavior::Wraparound) => {
				if transform.translation.x > WIDTH {
					transform.translation.x -= WIDTH * 2.
				}
				if transform.translation.x < -WIDTH {
					transform.translation.x += WIDTH * 2.
				}
				if transform.translation.y > HEIGHT {
					transform.translation.y -= HEIGHT * 2.
				}
				if transform.translation.y < -HEIGHT {
					transform.translation.y += HEIGHT * 2.
				}
			}
			Some(EdgeBehavior::Destroy) => {
				if transform.translation.x > WIDTH {
					cmds.entity(entity).despawn();
				}
				if transform.translation.x < -WIDTH {
					cmds.entity(entity).despawn();
				}
				if transform.translation.y > HEIGHT {
					cmds.entity(entity).despawn();
				}
				if transform.translation.y < -HEIGHT {
					cmds.entity(entity).despawn();
				}
			}
			None => (),
		}
	}
}

fn animations(mut animations: Query<(&mut Transform, &mut Animation<Transform>)>, time: Res<Time>) {
	for (mut t, mut anim) in animations.iter_mut() {
		anim.tick(time.delta_secs(), &mut t);
	}
}

fn y_sort(mut q: Query<&mut Transform, With<Sprite>>) {
	for mut t in q.iter_mut() {
		t.translation.z = -t.translation.y;
	}
}

fn handle_collisions(
	mut cmds: Commands,
	mut collisions: EventReader<CollisionEvent>,
	mut trader_changes: EventWriter<TraderChange>,
	mut spawn_events: EventWriter<SpawnProjectile>,
	mut trader: Query<&mut Trader>,
	rumor: Query<&Rumor>,
	trader_query: Query<(&Transform, Option<&TraderRestTimer>)>,
	projectile: Query<&Projectile>,
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
			if projectile.get(rumor_entity).unwrap().owner == Some(trader_entity)
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

			// TODO sometimes loop & crash if lots of stuff is overlapping. Owner not enough, need a disable timer
			// Spawn chain reaction bullets
			cmds.entity(rumor_entity).despawn();
			let position = trader_transform.translation.xy();
			spawn_events.write(SpawnProjectile {
				projectile_type: rumor,
				position,
				direction: Vec2::new(5., 0.),
				owner: Some(trader_entity),
			});
			spawn_events.write(SpawnProjectile {
				projectile_type: rumor,
				position,
				direction: Vec2::new(-5., 0.),
				owner: Some(trader_entity),
			});
			spawn_events.write(SpawnProjectile {
				projectile_type: rumor,
				position,
				direction: Vec2::new(0., 5.),
				owner: Some(trader_entity),
			});
			true
		};

		if is_rumor_trader {
			check_rumor_vs_trader(collision.entity1, collision.entity2);
		}
		if is_trader_rumor {
			check_rumor_vs_trader(collision.entity2, collision.entity1);
		}
	}
}

fn player_investing(key_input: Res<ButtonInput<KeyCode>>, mut stonks: ResMut<StonksTrading>) {
	if key_input.pressed(KeyCode::KeyB) {
		stonks.owned += 1;
		stonks.spent += stonks.price_current;
	}
	if key_input.just_pressed(KeyCode::KeyS) {
		stonks.returns_total += (stonks.owned * stonks.price_current) as i64 - stonks.spent as i64;
		stonks.owned = 0;
		stonks.spent = 0;
	}
}

fn player_shooting(
	key_input: Res<ButtonInput<KeyCode>>,
	mut spawn_events: EventWriter<SpawnProjectile>,
) {
	const START_POS: Vec2 = Vec2::new(0., -HEIGHT); // change later
	if key_input.just_pressed(KeyCode::Space) {
		spawn_events.write(SpawnProjectile {
			projectile_type: Rumor::Taco,
			position: START_POS,
			direction: Vec2::new(0., 5.),
			owner: None,
		});
	}
}

fn donnie_shooting(
	mut shooting_logic: ResMut<DonnieShootingLogic>,
	time: Res<Time>,
	mut spawn_events: EventWriter<SpawnProjectile>,
) {
	const START_POS: Vec2 = Vec2::new(0., HEIGHT); // change later
	if shooting_logic
		.shooting_timer
		.tick(time.delta())
		.just_finished()
	{
		spawn_events.write(SpawnProjectile {
			projectile_type: Rumor::Tariff,
			position: START_POS,
			direction: Vec2::new(0., -5.),
			owner: None,
		});
	}
}

fn update_trader_status(
	mut traders: Query<(&mut Sprite, &Trader)>,
	mut events: EventReader<TraderChange>,
	asset_server: Res<AssetServer>,
) {
	for event in events.read() {
		let (mut sprite, trader) = traders.get_mut(event.entity).unwrap();
		sprite.image = match trader.status {
			TraderStatus::Neutral => asset_server.load("ducky.png"),
			TraderStatus::Bearish => asset_server.load("bear-svgrepo-com.png"),
			TraderStatus::Bullish => asset_server.load("bull-svgrepo-com.png"),
		};
	}
}

fn tick_trader_timers(
	time: Res<Time>,
	mut query_status: Query<(&mut TraderStatusTimer, &mut Trader, Entity)>,
	query_rest: Query<(&mut TraderRestTimer, Entity)>,
	mut trader_changes: EventWriter<TraderChange>,
	mut cmds: Commands,
) {
	for (mut timer, mut trader, entity) in &mut query_status {
		if timer.tick(time.delta()).just_finished() {
			trader.status = TraderStatus::Neutral;
			trader_changes.write(TraderChange { entity });
			cmds.entity(entity).remove::<TraderStatusTimer>();
		}
	}

	for (mut timer, entity) in query_rest {
		if timer.tick(time.delta()).just_finished() {
			cmds.entity(entity).remove::<TraderRestTimer>();
		}
	}
}

fn spawn_projectiles(
	mut spawn_events: EventReader<SpawnProjectile>,
	mut commands: Commands,
	asset_server: Res<AssetServer>,
) {
	for event in spawn_events.read() {
		commands.spawn((
			Sprite {
				image: asset_server.load(match event.projectile_type {
					Rumor::Tariff => "pile-of-poo-svgrepo-com.png",
					Rumor::Taco => "taco-svgrepo-com.png",
				}),
				custom_size: Some(vec2(50., 50.)),
				..Default::default()
			},
			event.projectile_type,
			Transform {
				translation: (event.position, 0.).into(),
				..Default::default()
			},
			Collider { radius: 25. },
			PhysicsBody {
				velocity: event.direction,
				..Default::default()
			},
			EdgeBehavior::Destroy,
			Projectile { owner: event.owner },
			Animation::<Transform> {
				progress: 0.,
				animation_speed: 1.,
				animations: vec![AnimValue::new(|t, _, n| t.rotate_local_z(n), |_| 0.1)],
			},
		));
	}
}

fn update_stonks_price(mut stonks: ResMut<StonksTrading>, query: Query<&Trader>) {
	let counts = query
		.iter()
		.map(|t| t.status)
		.fold([0, 0, 0], |mut c, status| {
			c[status as usize] += 1;
			c
		});
	let price_current = STONKS_PER_NEUTRAL * counts[TraderStatus::Neutral as usize]
		+ STONKS_PER_BEARISH * counts[TraderStatus::Bearish as usize]
		+ STONKS_PER_BULLISH * counts[TraderStatus::Bullish as usize];
	stonks.price_current = price_current;

	if stonks.price_history.len() > 300 {
		stonks.price_history.pop_front();
	}
	stonks.price_history.push_back(price_current);
}
