use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use rand::prelude::*;

pub mod physics;

use physics::*;

const WIDTH: f32 = 300.;
const HEIGHT: f32 = 300.;

#[derive(Default, PartialEq)]
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

#[derive(Event)]
struct TraderChange {
    entity: Entity,
}

#[derive(Component, Deref, DerefMut)]
struct TraderStatusTimer(Timer);

#[derive(Component)]
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

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            // Wasm builds will check for meta files (that don't exist) if this isn't set.
            // This causes errors and even panics in web builds on itch.
            // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
            meta_check: AssetMetaCheck::Never,
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, (
            player_shooting,
            don_shooting,
            move_entities,
            check_collisions,
            handle_collisions,
            tick_trader_timers,
            update_trader_status,
        ).chain())
        .add_event::<CollisionEvent>()
        .add_event::<TraderChange>()
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    let mut rng = rand::rng();
    for _ in 0..10 {
        commands.spawn((
            Sprite {
                image: asset_server.load("ducky.png"),
                custom_size: Some(vec2(50., 50.)),
                ..Default::default()
            },
            Transform {
                translation: Vec3::new(rng.random_range(-WIDTH.. WIDTH), rng.random_range(-HEIGHT.. HEIGHT), 0.),
                ..Default::default()
            },
            Trader::default(),
            Collider {radius: 25.},
            PhysicsBody {
                velocity: Vec2::new(rng.random_range(-3.0.. 3.0), rng.random_range(-3.0.. 3.0)),
                ..Default::default()
            },
            RandomMovement,
            EdgeBehavior::Wraparound,
        ));
    }
}

fn move_entities(
    mut query: Query<(Entity, &mut PhysicsBody, &mut Transform, Option<&EdgeBehavior>)>,
    mut cmds: Commands,
) {
    for (entity, body, mut transform, maybe_edge) in query.iter_mut() {
        // transform.translation = ((transform.translation.xy() + body.velocity), 0.).into()
        transform.translation.x += body.velocity.x;
        transform.translation.y += body.velocity.y;

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
            },
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
            },
            None => (),
        }
    }
}

fn handle_collisions(
    mut collisions: EventReader<CollisionEvent>,
    mut trader_changes: EventWriter<TraderChange>,
    rumor: Query<&Rumor>,
    mut trader: Query<&mut Trader>,
    transform: Query<&Transform>,
    mut cmds: Commands,
    asset_server: Res<AssetServer>,
) {
    for collision in collisions.read() {
        let entity1_is_trader = trader.get(collision.entity1).is_ok();
        let entity2_is_trader = trader.get(collision.entity2).is_ok();

        let mut check_rumor_vs_trader = |rumor_entity, trader_entity| {
            let mut trader = trader.get_mut(trader_entity).unwrap();
            if trader.status == TraderStatus::Bullish {
                return false;
            }
            cmds.entity(rumor_entity).despawn();
            // TODO check rumor type
            trader.status = TraderStatus::Bullish;
            trader_changes.write(TraderChange {entity: trader_entity});
            cmds.entity(trader_entity).insert(TraderStatusTimer(Timer::from_seconds(5., TimerMode::Once)));
            spawn_taco(&mut cmds, &asset_server, transform.get(trader_entity).unwrap().translation.xy(), Vec2::new(5., 0.));
            true
        };
        if rumor.get(collision.entity1).is_ok() && entity2_is_trader {
            check_rumor_vs_trader(collision.entity1, collision.entity2);
        }
        if rumor.get(collision.entity2).is_ok() && entity1_is_trader {
            check_rumor_vs_trader(collision.entity2, collision.entity1);   
        }
    }
}

fn player_shooting(
    key_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    const START_POS: Vec2 = Vec2::new(0., -HEIGHT);
    if key_input.just_pressed(KeyCode::Space) {
        spawn_taco(&mut commands, &asset_server, START_POS, Vec2::new(0., 5.));
    }
}

fn don_shooting() {
    // TODO use timer
    // normalize bullet types through event
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
            TraderStatus::Bullish => asset_server.load("free-bull-svgrepo-com.png"),
        };
    }
}

fn tick_trader_timers(
    time: Res<Time>,
    mut query: Query<(&mut TraderStatusTimer, &mut Trader, Entity)>,
    mut trader_changes: EventWriter<TraderChange>,
) {
    for (mut timer, mut trader, entity) in &mut query {
        if timer.tick(time.delta()).just_finished() {
            trader.status = TraderStatus::Neutral;
            trader_changes.write(TraderChange {entity});
            // TODO despawn timer
        }
    }
}

// replace with an actual system?
fn spawn_taco(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    position: Vec2,
    velocity: Vec2
) {
    commands.spawn((
        Sprite {
            image: asset_server.load("taco-svgrepo-com.png"),
            custom_size: Some(vec2(50., 50.)),
            ..Default::default()
        },
        Rumor::Taco,
        Transform {
            translation: (position, 0.).into(),
            ..Default::default()
        },
        Collider {radius: 25.},
        PhysicsBody {
            velocity,
            ..Default::default()
        },
        EdgeBehavior::Destroy,
    ));
}