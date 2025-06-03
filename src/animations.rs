use bevy::prelude::*;

#[derive(Component)]
pub struct Animation<T> {
	pub progress: f32,
	pub animation_speed: f32,
	pub animations: Vec<AnimValue<T>>,
}

impl<T> Animation<T> {

	pub fn tick(&mut self, delta: f32, t: &mut T) {
		for anim in self.animations.iter() {
			// could cache prev_value
			let prev_value = (anim.val_f)(self.progress); 
			let new_value = (anim.val_f)(self.progress + delta * self.animation_speed);
			(anim.setter_f)(t, prev_value, new_value);
		}
		self.progress += delta * self.animation_speed;
	}
}

pub struct AnimValue<T> {
	pub val_f: fn(progress: f32) -> f32,
	pub setter_f: fn(t: &mut T, old: f32, new: f32),
}

impl<T> AnimValue<T> {
	pub fn new(
		setter_f: fn(t: &mut T, old: f32, new: f32),
		val_f: fn(progress: f32) -> f32,
	) -> Self {
		AnimValue { val_f, setter_f }
	}
}