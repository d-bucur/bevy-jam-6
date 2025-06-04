use std::ops::DerefMut;

use crate::*;

#[derive(Component)]
pub enum EdgeBehavior {
	Wraparound,
	Destroy,
}

#[derive(Component, Clone)]
pub enum RandomMovement {
	Moving(Timer),
	Idle(Timer),
}

impl Default for RandomMovement {
	fn default() -> Self {
		if rand::random_bool((IDLE_TIME / MOVEMENT_TIME) as f64) {
			Self::Idle(Timer::from_seconds(
				rand::random_range(0.0..IDLE_TIME),
				TimerMode::Once,
			))
		} else {
			Self::Moving(Timer::from_seconds(
				rand::random_range(0.0..MOVEMENT_TIME),
				TimerMode::Once,
			))
		}
	}
}

pub fn move_entities(
	mut query: Query<(
		Entity,
		&mut PhysicsBody,
		&mut Transform,
		Option<&EdgeBehavior>,
		Option<&mut Sprite>,
	)>,
	mut cmds: Commands,
) {
	for (entity, body, mut transform, maybe_edge, maybe_sprite) in query.iter_mut() {
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

pub fn animations(
	mut animations: Query<(&mut Transform, &mut Animation<Transform>)>,
	time: Res<Time>,
) {
	for (mut t, mut anim) in animations.iter_mut() {
		anim.tick(time.delta_secs(), &mut t);
	}
}

pub fn y_sort(mut q: Query<&mut Transform, With<Sprite>>) {
	for mut t in q.iter_mut() {
		t.translation.z = -t.translation.y;
	}
}

pub fn handle_random_movement(
	mut q: Query<(&mut PhysicsBody, &mut RandomMovement)>,
	time: Res<Time>,
) {
	for (mut body, mut random_movement) in q.iter_mut() {
		match random_movement.deref_mut() {
			RandomMovement::Moving(timer) => {
				if timer.tick(time.delta()).just_finished() {
					*random_movement =
						RandomMovement::Idle(Timer::from_seconds(IDLE_TIME, TimerMode::Once));
					body.velocity = Vec2::ZERO;
				}
			}
			RandomMovement::Idle(timer) => {
				if timer.tick(time.delta()).just_finished() {
					*random_movement =
						RandomMovement::Moving(Timer::from_seconds(MOVEMENT_TIME, TimerMode::Once));
					body.velocity = get_trader_random_velocity();
				}
			}
		};
	}
}
