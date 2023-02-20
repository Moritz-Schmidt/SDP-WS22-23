use crate::move_steering::MoveSteering;
use crate::sensor::LineSensor;
use crate::settings::Settings;

use ev3dev_lang_rust::motors::{LargeMotor};
use ev3dev_lang_rust::sensors::{UltrasonicSensor};

use ev3dev_lang_rust::Ev3Button;
use ev3dev_lang_rust::Ev3Result;


type TaskFunc<Res> = Box<dyn Fn(&mut Robot) -> Ev3Result<Res> + Send>;

pub struct Task {
    pub name: String,
    pub act: TaskFunc<()>,
    pub cond: TaskFunc<bool>,
}

impl Task {
    pub fn new(
        name: String,
        act: TaskFunc<()>,
        cond: TaskFunc<bool>,
    ) -> Self {
        Self { name, act, cond }
    }
}

pub struct Robot {
    pub steering: MoveSteering,
    pub left_sensor: LineSensor,
    pub right_sensor: LineSensor,
    pub middle_sensor: LineSensor,
    pub us_sensor: UltrasonicSensor,
    pub ball_motor: LargeMotor,
    pub speed: i32,
    pub max_steering: i32,
    pub settings: Settings,
}

impl Robot {
    pub fn new(
        steering: MoveSteering,
        left_sensor: LineSensor,
        right_sensor: LineSensor,
        middle_sensor: LineSensor,
        us_sensor: UltrasonicSensor,
        ball_motor: LargeMotor,
        settings: Settings,
    ) -> Self {
        Self {
            steering,
            left_sensor,
            right_sensor,
            middle_sensor,
            us_sensor,
            ball_motor,
            speed: settings.steering.speed,
            max_steering: settings.steering.max_steering,
            settings,
        }
    }

    pub fn forward(&mut self) -> Ev3Result<()> {
        self.steering.on(0, self.speed)?;
        Ok(())
    }

    pub fn turn_right_until_line(&mut self) -> Ev3Result<()> {
        self.steering.on(self.max_steering, self.speed)?;
        self.right_sensor.wait_for_line(true, std::time::Duration::from_millis(self.settings.steering.turn_timeout))?;
        //self.steering.off()?;
        Ok(())
    }

    pub fn turn_left_until_line(&mut self) -> Ev3Result<()> {
        self.steering.on(-self.max_steering, self.speed)?;
        self.left_sensor.wait_for_line(true, std::time::Duration::from_millis(self.settings.steering.turn_timeout))?;
        //self.steering.off()?;
        Ok(())
    }

    pub fn follow_line_loop(&mut self, task: Option<Task>) -> Ev3Result<()> {
        let button = Ev3Button::new()?;
        match task {
            Some(task) => loop {
                button.process();
                if !button.get_pressed_buttons().is_empty() {
                    self.steering.off()?;
                    break;
                }
                if (task.cond)(self).unwrap_or(false) {
                    println!("Task {} started", task.name);
                    (task.act)(self)?;
                    println!("Task {} finished", task.name);
                    break;
                }
                self.follow_line_once()?;
            },
            None => loop {
                button.process();
                if !button.get_pressed_buttons().is_empty() {
                    self.steering.off()?;
                    break;
                }
                self.follow_line_once()?;
            },
        }
        Ok(())
    }

    pub fn follow_line_once(&mut self) -> Ev3Result<()> {
        if self.left_sensor.off_line()
            && self.middle_sensor.on_line()
            && self.right_sensor.off_line()
        {
            self.forward()?;
        } else if self.left_sensor.off_line()
            && self.middle_sensor.on_line()
            && self.right_sensor.on_line()
        {
            self.turn_right_until_line()?;
            //self.steering.on(self.max_steering, self.speed)?;
        } else if self.left_sensor.on_line()
            && self.middle_sensor.on_line()
            && self.right_sensor.off_line()
        {
            self.turn_left_until_line()?;
            //self.steering.on(-self.max_steering, self.speed)?;
        } else if self.left_sensor.off_line()
            && self.middle_sensor.off_line()
            && self.right_sensor.off_line()
        {
            self.forward()?;
        } else if self.left_sensor.on_line()
            && self.middle_sensor.off_line()
            && self.right_sensor.off_line()
        {
            self.turn_left_until_line()?;
            //self.steering.on(-self.max_steering, self.speed)?;
        } else if self.left_sensor.off_line()
            && self.middle_sensor.off_line()
            && self.right_sensor.on_line()
        {
            self.turn_right_until_line()?;
            //self.steering.on(self.max_steering, self.speed)?;
        } else {
            self.forward()?;
        }
        Ok(())
    }
}
