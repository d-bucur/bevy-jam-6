use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use rand::prelude::*;

pub mod physics;

use physics::*;

const WIDTH: f32 = 300.;
const HEIGHT: f32 = 300.;

#[derive(Component)]
struct Trader;

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
            move_entities,
            check_collisions,
            handle_collisions,
        ).chain())
        .add_event::<CollisionEvent>()
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
            Trader,
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
    rumor: Query<&Rumor>,
    trader: Query<&Trader>,
    mut cmds: Commands,
) {
    for collision in collisions.read() {
        if rumor.get(collision.entity1).is_ok() && trader.get(collision.entity2).is_ok() {
            cmds.entity(collision.entity1).despawn();
        }
        if rumor.get(collision.entity2).is_ok() && trader.get(collision.entity1).is_ok() {
            cmds.entity(collision.entity2).despawn();
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
        commands.spawn((
            Sprite {
                image: asset_server.load("taco-svgrepo-com.png"),
                custom_size: Some(vec2(50., 50.)),
                ..Default::default()
            },
            Rumor::Taco,
            Transform {
                translation: (START_POS, 0.).into(),
                ..Default::default()
            },
            Collider {radius: 25.},
            PhysicsBody {
                velocity: Vec2::new(0., 5.),
                ..Default::default()
            },
            EdgeBehavior::Destroy,
        ));
    }
}