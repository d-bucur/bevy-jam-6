use std::f32::consts::PI;

use bevy::math::Vec2;

pub const GAME_NAME: &str = "Donnie's Tacos";
pub const WIDTH: f32 = 600.;
pub const HEIGHT: f32 = 350.;

pub const ROUND_TIME: f32 = 60.;

pub const STONKS_PER_BEARISH: u32 = 3;
pub const STONKS_PER_NEUTRAL: u32 = 5;
pub const STONKS_PER_BULLISH: u32 = 7;
pub const STONKS_DATA_POINTS: u32 = 300;
pub const STONKS_PER_BUY_ACTION: u32 = 300;

pub const TRADER_COUNT: u32 = 15;
pub const PROJECTILE_SPEED: f32 = 7.;
pub const MOVEMENT_TIME: f32 = 5.;
pub const IDLE_TIME: f32 = 1.;

pub const MAX_TACOS: u32 = 3;
pub const TACO_CHARGE_TIME: f32 = 1.;

pub const DONNIE_LINE_CHANCE: f64 = 0.5;
pub const DONNIE_LIE_CHANCE: f64 = 1.;

pub fn get_trader_random_velocity() -> Vec2 {
	const TRADER_MAX_VELOCITY: f32 = 2.0;
	let angle = rand::random_range(0.0..PI) * 2.;
	Vec2::new(angle.cos(), angle.sin()) * rand::random_range(0.5..1.0) * TRADER_MAX_VELOCITY
}

// computed from above
pub const PRICE_LOWEST: f32 = (STONKS_PER_BEARISH * TRADER_COUNT) as f32;
pub const PRICE_HIGHEST: f32 = (STONKS_PER_BULLISH * TRADER_COUNT) as f32;
