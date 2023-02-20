use config::Config;
use serde_derive::Deserialize;
use crate::move_steering::StopAction;
use std::error::Error;
use clap::builder::TypedValueParser as _;

#[derive(Debug, Deserialize)]
pub struct Steering {
    pub speed: i32,
    pub max_steering: i32,
    pub stop_action: StopAction,
    pub turn_timeout: u64,
}

#[derive(Debug, Deserialize)]
pub struct Sensors {
    pub left_threshold: i32,
    pub right_threshold: i32,
    pub middle_threshold: i32
}
#[derive(Debug, Deserialize)]
pub struct Turn {
    pub steering: i32,
    pub speed: i32,
    pub rotations: f32,
}
#[derive(Debug, Deserialize)]
pub struct WaitForBall {
    pub drive_until_dist: f32,
    pub finish_dist: f32,
}

#[derive(Debug, Deserialize)]
pub struct OnForRotations {
    pub steering: i32,
    pub speed: i32,
    pub rotations: f32
}

#[derive(Debug, Deserialize)]
pub struct Drive {
    pub steering: i32,
    pub speed: i32,
}

#[derive(Debug, Deserialize)]
pub struct PushBlock {
    pub drive_until_dist: f32,
    pub leave_line: UTurn,
    pub u_turn: UTurn,
    pub return_to_line: ReturnToLine,
}

#[derive(Debug, Deserialize)]
pub struct UTurn {
    pub on_for_rotations: OnForRotations,
    pub drive: Drive,
}

#[derive(Debug, Deserialize)]
pub struct ReturnToLine {
    pub steering: i32,
    pub speed: i32,
    pub seconds: i32,
}

#[derive(Debug, Deserialize)]
pub struct ThrowBall {
    pub drive_until_dist: f32,
    pub speed: i32,
    pub ball_motor_speed: i32,
    pub ball_motor_rel_pos: i32,
}

#[derive(Debug, Deserialize)]
pub struct Act {
    pub turn: Turn,
    pub wait_for_ball: WaitForBall,
    pub push_block: PushBlock,
    pub throw_ball: ThrowBall,
}

#[derive(Debug, Deserialize)]
pub struct Lines {
    pub time: i32,
    pub count: i32,
}

#[derive(Debug, Deserialize)]
pub struct Cond {
    pub lines: Lines,
    pub dist: f32,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub steering: Steering,
    pub sensors: Sensors,
    pub act: Act,
    pub cond: Cond,
}

impl Settings {
    pub fn new(overrides: Vec<Override>, file: std::path::PathBuf) ->  Result<Self, config::ConfigError> {
        let mut builder = Config::builder()
            .add_source(config::File::with_name("/home/robot/SDP2022/rs/settings/default.yaml"))
            .add_source(config::File::with_name(file.to_str().unwrap_or("")).required(false));
        for (key, value) in overrides {
            builder = builder.set_override(key, value)?;
        };
        let config = builder.build()?;
        config.try_deserialize()
    }
}

pub type Override = (String, String);

pub fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: Error + Send + Sync + 'static,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{}`", s))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}