use crate::*;

#[derive(Component, PartialEq, Clone, Copy)]
pub enum Rumor {
	Tariff,
	Taco,
}

#[derive(Event)]
pub struct SpawnProjectile {
	pub projectile_type: Rumor,
	pub position: Vec2,
	pub direction: Vec2,
	pub owner: Option<Entity>, // TODO replace with relationship
}

#[derive(Resource)]
pub struct DonnieShootingLogic {
	shooting_timer: Timer,
}

impl Default for DonnieShootingLogic {
	fn default() -> Self {
		Self {
			shooting_timer: Timer::from_seconds(2.0, TimerMode::Repeating),
		}
	}
}

#[derive(Component)]
pub struct Projectile {
	pub owner: Option<Entity>,
}

#[derive(Event)]
pub struct RumorJustShot;

#[derive(Component)]
pub struct PlayerShootingLogic {
	pub timer: Timer,
	pub tacos_left: u32,
	pub max_tacos: u32,
}

impl Default for PlayerShootingLogic {
	fn default() -> Self {
		Self {
			timer: Timer::from_seconds(TACO_CHARGE_TIME, TimerMode::Repeating),
			tacos_left: MAX_TACOS,
			max_tacos: MAX_TACOS,
		}
	}
}

pub trait BulletPattern {
	fn direction_iter(self, reference_dir: Vec2) -> impl Iterator<Item = Vec2>;
}

pub struct UniformPattern {
	pub bullet_count: u32,
}

impl BulletPattern for UniformPattern {
	fn direction_iter(self, reference_dir: Vec2) -> impl Iterator<Item = Vec2> {
		let angle_step = std::f32::consts::PI * 2. / self.bullet_count as f32;
		(0..self.bullet_count).map(move |i| {
			let angle = i as f32 * angle_step;
			Vec2::from_angle(angle).rotate(reference_dir)
		})
	}
}

// Too big. Should break up
pub fn player_shooting(
	key_button: Res<ButtonInput<KeyCode>>,
	mouse_button: Res<ButtonInput<MouseButton>>,
	mut spawn_events: EventWriter<SpawnProjectile>,
	mut gizmos: Gizmos<DottedGizmoConfig>,
	window: Single<&Window>,
	camera: Single<(&Camera, &GlobalTransform)>,
	player: Single<(&Transform, Entity), With<Player>>,
	mut shoot_logic: Single<&mut PlayerShootingLogic>,
	mut arrow: Single<&mut Transform, (With<PlayerArrowIndicator>, Without<Player>)>,
	mut stats: ResMut<GameStats>,
	mut cmds: Commands,
) {
	// draw shooting line
	let Some(cursor_pos) = window
		.cursor_position()
		.and_then(|p| camera.0.viewport_to_world_2d(camera.1, p).ok())
	else {
		return;
	};

	// not sure if should use arrow or gizmo line. keeping both for now
	let start_pos: Vec2 = player.0.translation.xy();
	gizmos.line_2d(
		start_pos,
		cursor_pos,
		bevy::color::palettes::css::YELLOW.with_alpha(0.5),
	);
	const ARROW_DISTANCE: f32 = 100.;
	let dir = (cursor_pos - start_pos).normalize();
	arrow.translation = (start_pos + dir * ARROW_DISTANCE).extend(900.);
	arrow.rotation = Quat::from_rotation_z(dir.to_angle());

	// fire taco
	if shoot_logic.tacos_left == 0 {
		return;
	}
	if mouse_button.just_pressed(MouseButton::Left) {
		spawn_events.write(SpawnProjectile {
			projectile_type: Rumor::Taco,
			position: start_pos,
			direction: (cursor_pos - start_pos).normalize() * PROJECTILE_SPEED,
			owner: None,
		});
		shoot_logic.tacos_left -= 1;
		cmds.trigger_targets(RumorJustShot, player.1);
	}
}

pub fn charge_player_tacos(mut q: Single<&mut PlayerShootingLogic>, time: Res<Time>) {
	if q.tacos_left >= q.max_tacos {
		return;
	}
	if q.timer.tick(time.delta()).just_finished() {
		q.tacos_left = (q.tacos_left + 1).clamp(0, q.max_tacos);
	}
}

pub fn donnie_shooting(
	// TODO should combine Donnie with DonnieShootingLogic?
	mut query: Query<(&Transform, Entity, &mut Sprite), With<Donnie>>,
	mut shooting_logic: ResMut<DonnieShootingLogic>,
	traders_q: Query<&Transform, With<Trader>>,
	time: Res<Time>,
	mut spawn_events: EventWriter<SpawnProjectile>,
	mut overhead_events: EventWriter<OverheadTextRequest>,
	asset_server: Res<AssetServer>,
	mut cmds: Commands,
) {
	if !shooting_logic
		.shooting_timer
		.tick(time.delta())
		.just_finished()
	{
		return;
	}
	use rand::seq::IteratorRandom;
	let (transform, entity, mut sprite) = query.single_mut().unwrap();
	let mut rng = rand::rng();
	let direction = traders_q
		.iter()
		.choose(&mut rng)
		.map(|trader| (trader.translation.xy() - transform.translation.xy()).normalize())
		.unwrap_or(Vec2::new(0., -1.));

	spawn_events.write(SpawnProjectile {
		projectile_type: Rumor::Tariff,
		position: transform.translation.xy(),
		direction: direction * PROJECTILE_SPEED,
		owner: Some(entity),
	});
	overhead_events.write(OverheadTextRequest {
		attached_to: entity,
		text: Some(random_tariff()),
		duration_sec: Some(1.5),
	});
	sprite.image = asset_server.load(donnie_texture_path());
	cmds.trigger_targets(RumorJustShot, entity);
}

pub fn spawn_projectiles(
	mut spawn_events: EventReader<SpawnProjectile>,
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	mut stats: ResMut<GameStats>,
) {
	for event in spawn_events.read() {
		commands.spawn((
			Sprite {
				image: asset_server.load(match event.projectile_type {
					Rumor::Tariff => "pile-of-poo-svgrepo-com.png",
					Rumor::Taco => "taco_man3/taco.png",
				}),
				custom_size: Some(vec2(50., 50.)),
				..Default::default()
			},
			event.projectile_type,
			Transform {
				translation: (event.position, 0.).into(),
				..Default::default()
			},
			Collider {
				radius: 20.,
				offset: Vec2::ZERO,
			},
			PhysicsBody {
				velocity: event.direction,
				..Default::default()
			},
			EdgeBehavior::Destroy,
			Projectile { owner: event.owner },
			AreaTrigger,
			Animation::<Transform> {
				progress: 0.,
				animation_speed: 1.,
				animations: vec![AnimValue::new(|t, _, n| t.rotate_local_z(n), |_| 0.1)],
			},
		));
		stats.total_projectiles_launched += 1;
	}
}
