use ev3dev_lang_rust::motors::{LargeMotor};
use ev3dev_lang_rust::Ev3Result;
use clap;
use serde_derive::Deserialize;

#[derive(clap::ValueEnum, Clone, Copy, Debug, Deserialize)]
pub enum StopAction {
    COAST,
    BRAKE,
    HOLD,
}

impl StopAction {
    pub fn to_str(&self) -> &str {
        match self {
            StopAction::COAST => LargeMotor::STOP_ACTION_COAST,
            StopAction::BRAKE => LargeMotor::STOP_ACTION_BRAKE,
            StopAction::HOLD => LargeMotor::STOP_ACTION_HOLD,
        }
    }
}
pub struct MoveSteering {
    pub left_motor: LargeMotor,
    pub right_motor: LargeMotor,
    pub max_speed: i32,
}

impl MoveSteering {
    pub fn new(
        left_motor: LargeMotor,
        right_motor: LargeMotor,
        stop_action: StopAction,
    ) -> Ev3Result<Self> {
        left_motor.set_stop_action(stop_action.to_str())?;
        right_motor.set_stop_action(stop_action.to_str())?;
        left_motor.set_polarity(LargeMotor::POLARITY_NORMAL)?;
        right_motor.set_polarity(LargeMotor::POLARITY_NORMAL)?;
        let max_speed = left_motor.get_max_speed().unwrap_or(0);

        Ok(Self {
            left_motor,
            right_motor,
            max_speed,
        })
    }

    pub fn get_speed_steering(&self, steering: i32, speed: i32) -> (i32, i32) {
        let speed: f32 = speed as f32 / 100.0 * self.max_speed as f32;
        let speed_factor = (50.0 - steering.abs() as f32) / 50.0;
        if steering >= 0 {
            (speed as i32, (speed * speed_factor) as i32)
        } else {
            ((speed * speed_factor) as i32, speed as i32)
        }
    }

    pub fn set_speed_sp(&self, speed: (i32, i32)) -> Ev3Result<()> {
        self.left_motor.set_speed_sp(speed.0)?;
        self.right_motor.set_speed_sp(speed.1)?;
        Ok(())
    }

    pub fn run_forever(&self) -> Ev3Result<()> {
        self.left_motor.run_forever()?;
        self.right_motor.run_forever()?;
        Ok(())
    }

    pub fn run_timed(&self, duration: std::time::Duration) -> Ev3Result<()> {
        self.left_motor.run_timed(Some(duration))?;
        self.right_motor.run_timed(Some(duration))?;
        Ok(())
    }

    pub fn off(&self) -> Ev3Result<()> {
        self.left_motor.stop()?;
        self.right_motor.stop()?;
        self.left_motor.wait_until_not_moving(None);
        self.right_motor.wait_until_not_moving(None);
        Ok(())
    }

    pub fn on(&self, steering: i32, speed: i32) -> Ev3Result<()> {
        let speeds = self.get_speed_steering(steering, speed);
        self.set_speed_sp(speeds)?;
        self.run_forever()?;
        Ok(())
    }

    pub fn on_for_seconds(
        &self,
        steering: i32,
        speed: i32,
        seconds: i32,
        block: bool,
    ) -> Ev3Result<()> {
        let speeds = self.get_speed_steering(steering, speed);
        let duration = std::time::Duration::from_secs(seconds as u64);
        self.set_speed_sp(speeds)?;
        self.run_timed(duration)?;
        if block {
            std::thread::sleep(duration);
            self.off()?;
        }
        Ok(())
    }

    pub fn get_rel_pos_and_speed(&self, speed: i32, mut degrees: f32) -> (i32, i32) {
        if speed < 0 {
            degrees *= -1.0;
        }
        let pos_delta =
            (degrees as i32 * self.left_motor.get_count_per_rot().unwrap_or(0)) / 360;
        (pos_delta, speed.abs())
    }

    pub fn on_for_rotations(
        &self,
        steering: i32,
        speed: i32,
        rotations: f32,
        block: bool,
    ) -> Ev3Result<()> {
        let speeds = self.get_speed_steering(steering, speed);
        let degrees = rotations * 360.0;
        let (left_degrees, right_degrees) =
            if degrees == 0.0_f32 || speeds.0 == 0 && speeds.1 == 0 {
                (degrees, degrees)
            } else if speeds.0.abs() > speeds.1.abs() {
                (degrees, (speeds.0 as f32 / speeds.1 as f32).abs() * degrees)
            } else {
                ((speeds.1 as f32 / speeds.0 as f32).abs() * degrees, degrees)
            };
        let left_delta = self.get_rel_pos_and_speed(speeds.0, left_degrees);
        let right_delta = self.get_rel_pos_and_speed(speeds.1, right_degrees);
        self.set_speed_sp((left_delta.1, right_delta.1))?;
        self.left_motor.run_to_rel_pos(Some(left_delta.0))?;
        self.right_motor.run_to_rel_pos(Some(right_delta.0))?;
        if block {
            self.left_motor.wait_until_not_moving(None);
            self.right_motor.wait_until_not_moving(None);
            self.off()?;
        }
        Ok(())
    }

    pub fn wait_until_not_moving(&self) -> Ev3Result<()> {
        self.left_motor.wait_until_not_moving(None);
        self.right_motor.wait_until_not_moving(None);
        Ok(())
    }
}