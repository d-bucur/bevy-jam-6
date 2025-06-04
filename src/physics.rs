use bevy::{color::palettes::css::*, prelude::*};

#[derive(Component, Default)]
pub struct Collider {
	pub radius: f32,
	pub offset: Vec2,
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
	// pub accel: Vec2,
}

#[derive(Event)]
pub struct CollisionEvent {
	pub entity1: Entity,
	pub entity2: Entity,
}

pub fn check_collisions(
	mut query: Query<(
		&mut PhysicsBody,
		&Collider,
		&mut Transform,
		Entity,
		Option<&AreaTrigger>,
	)>,
	mut collisions: EventWriter<CollisionEvent>,
) {
	let mut combinations = query.iter_combinations_mut();
	while let Some([e1, e2]) = combinations.fetch_next() {
		let axis = (e1.2.translation.xy() + e1.1.offset) - (e2.2.translation.xy() + e2.1.offset);
		if axis.length() < e1.1.radius + e2.1.radius {
			// handle collision
			collisions.write(CollisionEvent {
				entity1: e1.3,
				entity2: e2.3,
			});
		}
	}
}

pub fn debug_colliders(mut gizmos: Gizmos, query: Query<(&Transform, &Collider), With<Collider>>) {
	for (transform, collider) in query.iter() {
		gizmos.circle_2d(
			Isometry2d::from_translation(transform.translation.xy() + collider.offset),
			collider.radius,
			BLUE.with_alpha(0.5),
		);
	}
}
