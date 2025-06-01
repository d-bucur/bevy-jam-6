use bevy::prelude::*;

#[derive(Component, Default)]
pub struct Collider {
    pub radius: f32,
}

/// Does not displace bodies on collision
// TODO not used yet
#[derive(Component)]
#[require(Collider)]
pub struct AreaTrigger;

#[derive(Component, Default)]
#[require(Transform)]
pub struct PhysicsBody {
    pub velocity: Vec2,
    pub accel: Vec2,
}

#[derive(Event)]
pub struct CollisionEvent {
	pub entity1: Entity,
	pub entity2: Entity,
}

pub fn check_collisions (
    mut query: Query<(&mut PhysicsBody, &Collider, &mut Transform, Entity, Option<&AreaTrigger>)>,
    mut collisions: EventWriter<CollisionEvent>,
) {
    let mut combinations = query.iter_combinations_mut();
    while let Some([mut e1, mut e2]) = combinations.fetch_next() {
        let axis = e1.2.translation - e2.2.translation;
        if axis.length() < e1.1.radius + e2.1.radius {
            // handle collision
            collisions.write(CollisionEvent {
                entity1: e1.3,
                entity2: e2.3,
            });
        }
    }
}