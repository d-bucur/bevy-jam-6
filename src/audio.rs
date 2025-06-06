use crate::*;
use bevy::{
	audio::*,
	ecs::{component::HookContext, world::DeferredWorld},
	platform::collections::HashMap,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum AudioType {
	DonnieVoice,
	TraderStatusChange,
	ProjectileShot,
	StonksNotifcation,
	Music,
}

#[derive(Resource, Deref, DerefMut)]
pub struct AudioLimitCounters(pub [u32; 4]);

#[derive(Component)]
#[component(on_add = on_audio_type_added)]
#[component(on_remove = on_audio_type_removed)]
struct LimitedAudio(AudioType);

#[derive(Resource, Deref)]
pub struct VolumeSettings {
	per_channel: HashMap<AudioType, f32>,
}

impl Default for VolumeSettings {
	fn default() -> Self {
		const DEFAULT_VOLUME: f32 = 1.0;
		Self {
			per_channel: [
				(AudioType::DonnieVoice, DEFAULT_VOLUME),
				(AudioType::TraderStatusChange, DEFAULT_VOLUME),
				(AudioType::ProjectileShot, DEFAULT_VOLUME),
				(AudioType::StonksNotifcation, DEFAULT_VOLUME),
				(AudioType::Music, DEFAULT_VOLUME),
			]
			.into(),
		}
	}
}

fn on_audio_type_added(mut world: DeferredWorld, ctx: HookContext) {
	let audio_type = world.get::<LimitedAudio>(ctx.entity).unwrap().0;
	let mut counters = world.get_resource_mut::<AudioLimitCounters>().unwrap();
	counters[audio_type as usize] -= 1;
}

fn on_audio_type_removed(mut world: DeferredWorld, ctx: HookContext) {
	let audio_type = world.get::<LimitedAudio>(ctx.entity).unwrap().0;
	let mut counters = world.get_resource_mut::<AudioLimitCounters>().unwrap();
	counters[audio_type as usize] += 1;
}

pub fn setup_audio(
	asset_server: Res<AssetServer>,
	mut commands: Commands,
	volume: Res<VolumeSettings>,
) {
	// soundtrack
	commands.spawn((
		AudioPlayer::new(asset_server.load("audio/music/1161090_Funny-Cat.mp3")),
		PlaybackSettings {
			mode: PlaybackMode::Loop,
			volume: Volume::Linear(1.0 * volume[&AudioType::Music]),
			..default()
		},
	));
}

pub fn on_donnie_shot(
	trigger: Trigger<RumorJustShot>,
	mut cmds: Commands,
	asset_server: ResMut<AssetServer>,
	audio_counters: Res<AudioLimitCounters>,
	volume: Res<VolumeSettings>,
) {
	// info!("audio_projectile_shot.target: {:?}", trigger.target());
	if audio_counters[AudioType::DonnieVoice as usize] == 0 || rand::random_bool(0.7) {
		return;
	}
	cmds.spawn((
		AudioPlayer::new(asset_server.load(random_string(&DONNIE_VOICE_LINES))),
		PlaybackSettings {
			mode: PlaybackMode::Despawn,
			volume: Volume::Linear(0.7 * volume[&AudioType::DonnieVoice]),
			..default()
		},
		LimitedAudio(AudioType::DonnieVoice),
	));
}

pub fn on_trader_status_change(
	trigger: Trigger<TraderChange>,
	asset_server: ResMut<AssetServer>,
	mut cmds: Commands,
	audio_counters: Res<AudioLimitCounters>,
	volume: Res<VolumeSettings>,
) {
	if audio_counters[AudioType::TraderStatusChange as usize] == 0 {
		return;
	}
	// warn!("on_trader_status_change: {:?}", trigger.new);
	if trigger.new != TraderStatus::Neutral {
		let path = if trigger.new == TraderStatus::Bearish {
			random_string(&BEARISH)
		} else {
			random_string(&BULLISH)
		};
		cmds.spawn((
			AudioPlayer::new(asset_server.load(path)),
			PlaybackSettings {
				mode: PlaybackMode::Despawn,
				volume: Volume::Linear(1. * volume[&AudioType::TraderStatusChange]),
				..default()
			},
			LimitedAudio(AudioType::TraderStatusChange),
		));
	}
}

pub fn on_projectile_shot(
	trigger: Trigger<RumorJustShot>,
	asset_server: ResMut<AssetServer>,
	mut cmds: Commands,
	audio_counters: Res<AudioLimitCounters>,
	volume: Res<VolumeSettings>,
) {
	if audio_counters[AudioType::ProjectileShot as usize] == 0 {
		return;
	}
	cmds.spawn((
		AudioPlayer::new(asset_server.load(random_string(&PLOPS))),
		PlaybackSettings {
			mode: PlaybackMode::Despawn,
			volume: Volume::Linear(1. * volume[&AudioType::ProjectileShot]),
			..default()
		},
		LimitedAudio(AudioType::ProjectileShot),
	));
}

pub fn on_stonks_notification(
	trigger: Trigger<StonksPriceNotification>,
	asset_server: ResMut<AssetServer>,
	mut cmds: Commands,
	audio_counters: Res<AudioLimitCounters>,
	volume: Res<VolumeSettings>,
) {
	if audio_counters[AudioType::StonksNotifcation as usize] == 0 {
		return;
	}
	cmds.spawn((
		AudioPlayer::new(asset_server.load(random_string(match trigger.event() {
			StonksPriceNotification::LOW => &SCREAMS,
			StonksPriceNotification::HIGH => &RELIEF,
		}))),
		PlaybackSettings {
			mode: PlaybackMode::Despawn,
			volume: Volume::Linear(
				volume[&AudioType::StonksNotifcation]
					* match trigger.event() {
						StonksPriceNotification::LOW => 0.5,
						StonksPriceNotification::HIGH => 0.9,
					},
			),
			..default()
		},
		LimitedAudio(AudioType::StonksNotifcation),
	));
}

const DONNIE_VOICE_LINES: [&str; 3] = [
	"audio/soundboard/Voicy_We have a president who doesn't have a clue.mp3",
	"audio/soundboard/Voicy_Well i don't have to really get into specifics.mp3",
	"audio/soundboard/Voicy_Don't know what there doing.mp3",
];

const SCREAMS: [&str; 3] = [
	"audio/fx/9704__lithe-fider__fl_scream-2.wav",
	"audio/fx/9705__lithe-fider__fl_scream-3.wav",
	"audio/fx/9706__lithe-fider__fl_scream-4.wav",
];

const RELIEF: [&str; 1] = ["audio/fx/758831__universfield__comedic.mp3"];

const BULLISH: [&str; 1] = ["audio/fx/331381__qubodup__public-domain-jump-sound.wav"];

const BEARISH: [&str; 1] = ["audio/fx/423526__ccolbert70eagles23__karate-chop.m4a"];

const PLOPS: [&str; 1] = ["audio/fx/245645__unfa__cartoon-pop-clean.flac"];
