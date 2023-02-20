use ev3dev_lang_rust::sensors::{ColorSensor, LightSensor, Sensor};
use std::time::SystemTime;
use ev3dev_lang_rust::Ev3Result;

pub enum LineSensorType {
    Light(LightSensor),
    Color(ColorSensor),
}

pub struct LineSensor {
    threshold: i32,
    has_history: bool,
    pub history: [(Option<bool>, Option<SystemTime>); 10],
    sensor: LineSensorType,
}

impl LineSensor {
    pub fn new(sensor: LineSensorType, threshold: i32, has_history: bool) -> Ev3Result<Self> {
        match sensor {
            LineSensorType::Light(ref sensor) => {
                sensor.set_mode_reflect()?;
            }
            LineSensorType::Color(ref sensor) => {
                sensor.set_mode("COL-REFLECT")?;
            }
        };
        Ok(Self {
            threshold,
            has_history,
            history: [(None, None); 10],
            sensor,
        })
    }

    pub fn reflected_light_intensity(&self) -> i32 {
        match &self.sensor {
            LineSensorType::Light(sensor) => sensor.get_value(0).unwrap_or(0),
            LineSensorType::Color(sensor) => sensor.get_value(0).unwrap_or(0),
        }
    }

    pub fn on_line(&mut self) -> bool {
        let val = self.reflected_light_intensity();
        let res = val < self.threshold;
        if self.has_history && self.history[0].0 != Some(res) {
            self.history.rotate_right(1);
            self.history[0] = (Some(res), Some(SystemTime::now()));
        }
        res
    }

    pub fn off_line(&mut self) -> bool {
        !self.on_line()
    }

    pub fn wait_for_line(&mut self, on_line: bool, timeout: std::time::Duration) -> Ev3Result<()> {
        let started = SystemTime::now();
        while self.on_line() == on_line && started.elapsed().unwrap_or(timeout) < timeout {
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        Ok(())
    }
}
