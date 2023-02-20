extern crate ev3dev_lang_rust;
extern crate serde;
extern crate serde_derive;
pub mod move_steering;
pub mod robo;
pub mod sensor;
pub mod settings;
use clap::Parser;
use move_steering::MoveSteering;
use robo::{Robot, Task};
use sensor::{LineSensor, LineSensorType};

use ev3dev_lang_rust::motors::{LargeMotor, MotorPort};
use ev3dev_lang_rust::sensors::{ColorSensor, LightSensor, Sensor, SensorPort, UltrasonicSensor};
use ev3dev_lang_rust::sound;
use ev3dev_lang_rust::Ev3Button;
use ev3dev_lang_rust::Ev3Result;
use ev3dev_lang_rust::PowerSupply;
use rumqttc::{Client, MqttOptions, QoS};

fn calibrate_sensors() -> Ev3Result<(i32, i32, i32)> {
    let left_sensor = LightSensor::get(SensorPort::In1)?;
    let right_sensor = LightSensor::get(SensorPort::In4)?;
    let middle_sensor = ColorSensor::get(SensorPort::In3)?;
    left_sensor.set_mode_reflect()?;
    right_sensor.set_mode_reflect()?;
    middle_sensor.set_mode(ColorSensor::MODE_COL_REFLECT)?;
    let button = Ev3Button::new()?;
    let mut lsv_l: Vec<i32> = Vec::new();
    let mut msv_l: Vec<i32> = Vec::new();
    let mut rsv_l: Vec<i32> = Vec::new();
    println!("put sensors on line");
    println!("press any button to continue");
    sound::beep()?;
    button.process();
    while button.get_pressed_buttons().is_empty() {
        button.process();
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    for _ in 0..100 {
        lsv_l.push(left_sensor.get_value0()?);
        msv_l.push(middle_sensor.get_value0()?);
        rsv_l.push(right_sensor.get_value0()?);
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    let mut lsv_w: Vec<i32> = Vec::new();
    let mut msv_w: Vec<i32> = Vec::new();
    let mut rsv_w: Vec<i32> = Vec::new();
    println!("put sensors off line");
    println!("press any button to continue");
    sound::beep()?;
    button.process();
    while button.get_pressed_buttons().is_empty() {
        button.process();
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    for _ in 0..100 {
        lsv_w.push(left_sensor.get_value0()?);
        msv_w.push(middle_sensor.get_value0()?);
        rsv_w.push(right_sensor.get_value0()?);
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    let left_sensor_threshold = (lsv_l.iter().sum::<i32>() / lsv_l.len() as i32
        + lsv_w.iter().sum::<i32>() / lsv_w.len() as i32)
        / 2;
    let middle_sensor_threshold = (msv_l.iter().sum::<i32>() / msv_l.len() as i32
        + msv_w.iter().sum::<i32>() / msv_w.len() as i32)
        / 2;
    let right_sensor_threshold = (rsv_l.iter().sum::<i32>() / rsv_l.len() as i32
        + rsv_w.iter().sum::<i32>() / rsv_w.len() as i32)
        / 2;
    sound::beep()?;
    sound::beep()?;
    println!("got thresholds");
    println!("left: {}", left_sensor_threshold);
    println!("middle: {}", middle_sensor_threshold);
    println!("right: {}", right_sensor_threshold);
    Ok((
        left_sensor_threshold,
        middle_sensor_threshold,
        right_sensor_threshold,
    ))
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// settings file
    #[arg(long, value_hint = clap::ValueHint::FilePath)]
    settings: Option<std::path::PathBuf>,

    /// settings overrides
    /// format: key=value
    /// example: --override left_sensor_threshold=100
    #[arg(long, short, value_parser = settings::parse_key_val::<String, String>, action = clap::ArgAction::Append)]
    override_: Vec<(String, String)>,

    /// calibrate sensors
    #[arg(long, action = clap::ArgAction::SetTrue)]
    calibrate: bool,

    /// execute turn action
    #[arg(long, action = clap::ArgAction::SetTrue)]
    turn: bool,

    /// execute wait for ball action
    #[arg(long, action = clap::ArgAction::SetTrue)]
    wait_for_ball: bool,

    /// execute push block action
    #[arg(long, action = clap::ArgAction::SetTrue)]
    push_block: bool,

    /// execute throw ball action
    #[arg(long, action = clap::ArgAction::SetTrue)]
    throw_ball: bool,

    /// execute all actions (turn, wait for ball, push block, throw ball)
    #[arg(long, action = clap::ArgAction::SetTrue)]
    all: bool,

    /// exevute no actions
    #[arg(long, action = clap::ArgAction::SetTrue)]
    none: bool,

    /// publish sensor values to mqtt
    #[arg(long, action = clap::ArgAction::SetTrue)]
    mqtt: bool,

    /// mqtt address
    #[arg(long)]
    mqtt_address: Option<String>,

    /// stop
    #[arg(long, action = clap::ArgAction::SetTrue)]
    stop: bool,

    /// stop when distance is less than 20 cm
    #[arg(long, action = clap::ArgAction::SetTrue)]
    stop_dist: bool,

    /// party
    #[arg(long, action = clap::ArgAction::SetTrue)]
    party: bool,
}

fn act_turn(robo: &mut Robot) -> Ev3Result<()> {
    robo.steering.on_for_rotations(
        robo.settings.act.turn.steering,
        robo.settings.act.turn.speed,
        robo.settings.act.turn.rotations,
        true,
    )?;
    robo.steering.on(
        robo.settings.act.turn.steering,
        robo.settings.act.turn.speed,
    )?;
    robo.middle_sensor
        .wait_for_line(false, std::time::Duration::from_millis(500))?;
    robo.steering.off()?;
    Ok(())
}

fn cond_dist(robo: &mut Robot) -> Ev3Result<bool> {
    Ok(robo.us_sensor.get_distance_centimeters()? < robo.settings.cond.dist)
}

fn act_wait_for_ball(robo: &mut Robot) -> Ev3Result<()> {
    while robo.us_sensor.get_distance_centimeters()?
        > robo.settings.act.wait_for_ball.drive_until_dist
    {
        robo.follow_line_once()?;
    }
    robo.steering.off()?;
    robo.steering.wait_until_not_moving()?;
    while robo.us_sensor.get_distance_centimeters()? < robo.settings.act.wait_for_ball.finish_dist {
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    Ok(())
}

fn cond_lines(robo: &mut Robot) -> Ev3Result<bool> {
    Ok(robo
        .middle_sensor
        .history
        .iter()
        .filter(|&x| match x.1 {
            Some(x) => {
                x.elapsed().unwrap()
                    < std::time::Duration::from_millis(
                        u64::try_from(robo.settings.cond.lines.time).unwrap(),
                    )
            }
            None => false,
        })
        .count()
        > robo.settings.cond.lines.count as usize)
}

fn act_push_block(robo: &mut Robot) -> Ev3Result<()> {
    // drive 90° to the right
    robo.steering.on_for_rotations(
        robo.settings
            .act
            .push_block
            .leave_line
            .on_for_rotations
            .steering,
        robo.settings
            .act
            .push_block
            .leave_line
            .on_for_rotations
            .speed,
        robo.settings
            .act
            .push_block
            .leave_line
            .on_for_rotations
            .rotations,
        true,
    )?;
    //std::thread::sleep(std::time::Duration::from_millis(1000));
    robo.steering.on(
        robo.settings.act.push_block.leave_line.drive.steering,
        robo.settings.act.push_block.leave_line.drive.speed,
    )?;
    robo.middle_sensor
        .wait_for_line(false, std::time::Duration::from_millis(250))?;
    robo.steering.off()?;
    //std::thread::sleep(std::time::Duration::from_millis(3000));
    // drive to the right until the line is lost
    while robo.us_sensor.get_distance_centimeters()? > robo.settings.act.push_block.drive_until_dist
    {
        robo.follow_line_once()?;
    }
    //robo.steering.off()?;
    println!("pushing block");
    //std::thread::sleep(std::time::Duration::from_millis(100));
    while robo.us_sensor.get_distance_centimeters()?
        <= robo.settings.act.push_block.drive_until_dist
    {
        robo.follow_line_once()?;
    }
    robo.steering.off()?;
    println!("block pushed");
    std::thread::sleep(std::time::Duration::from_millis(100));
    println!("turning");
    // 180° turn
    robo.steering.on_for_rotations(
        robo.settings
            .act
            .push_block
            .u_turn
            .on_for_rotations
            .steering,
        robo.settings.act.push_block.u_turn.on_for_rotations.speed,
        robo.settings
            .act
            .push_block
            .u_turn
            .on_for_rotations
            .rotations,
        true,
    )?;
    robo.steering.on(
        robo.settings.act.push_block.u_turn.drive.steering,
        robo.settings.act.push_block.u_turn.drive.speed,
    )?;
    robo.middle_sensor
        .wait_for_line(false, std::time::Duration::from_millis(500))?;
    //robo.steering.off()?;
    println!("turning done");
    //std::thread::sleep(std::time::Duration::from_millis(3000));
    println!("drive until line lost");
    // drive to the right until the line is lost
    while robo.left_sensor.on_line() || robo.middle_sensor.on_line() || robo.right_sensor.on_line()
    {
        robo.follow_line_once()?;
    }
    //robo.steering.off()?;
    println!("line lost");
    //std::thread::sleep(std::time::Duration::from_millis(3000));

    // drive 90° to the right
    robo.steering.on_for_seconds(
        robo.settings.act.push_block.return_to_line.steering,
        robo.settings.act.push_block.return_to_line.speed,
        robo.settings.act.push_block.return_to_line.seconds,
        true
    )?;
    //println!("on for seconds done in:");
    //println!("{}", started.elapsed().as_millis());
    //std::thread::sleep(std::time::Duration::from_millis(20));
    //robo.middle_sensor
    //    .wait_for_line(false, std::time::Duration::from_millis(1000))?;
    //robo.steering.off()?;
    Ok(())
}

fn act_stop(robo: &mut Robot) -> Ev3Result<()> {
    robo.steering.off()?;
    Ok(())
}

fn act_throw_ball(robo: &mut Robot) -> Ev3Result<()> {
    while robo.us_sensor.get_distance_centimeters()? > robo.settings.act.throw_ball.drive_until_dist
    {
        robo.follow_line_once()?;
    }
    robo.steering.on(0, robo.settings.act.throw_ball.speed)?;
    robo.steering.left_motor.wait_until(
        LargeMotor::STATE_STALLED,
        Some(std::time::Duration::from_millis(250)),
    );
    robo.steering.off()?;

    robo.ball_motor.run_forever()?;
    robo.ball_motor
        .set_stop_action(LargeMotor::STOP_ACTION_HOLD)?;
    robo.ball_motor
        .set_speed_sp(robo.settings.act.throw_ball.ball_motor_speed)?;
    robo.ball_motor
        .run_to_rel_pos(Some(robo.settings.act.throw_ball.ball_motor_rel_pos))?;
    robo.ball_motor
        .wait_until_not_moving(Some(std::time::Duration::from_secs(2)));
    robo.ball_motor
        .run_to_rel_pos(Some(-robo.settings.act.throw_ball.ball_motor_rel_pos))?;
    Ok(())
}

fn cond_party(robo: &mut Robot) -> Ev3Result<bool> {
    Ok(true)
}

fn act_party(robo: &mut Robot) -> Ev3Result<()> {
    //sound::play("/home/robot/SDP2022/win.wav")?.wait()?;
    //robo.steering.on_for_rotations(0, -100, 1.0, true)?;
    robo.steering.on(100, 100)?;
    robo.ball_motor
        .set_speed_sp(900)?;
    let started = std::time::Instant::now();
    while started.elapsed() < std::time::Duration::from_secs(20) {
        robo.ball_motor
            .wait_until_not_moving(Some(std::time::Duration::from_millis(500)));
        robo.ball_motor
            .run_to_rel_pos(Some(robo.settings.act.throw_ball.ball_motor_rel_pos))?;
        robo.ball_motor
            .wait_until_not_moving(Some(std::time::Duration::from_millis(500)));
        robo.ball_motor
            .run_to_rel_pos(Some(-robo.settings.act.throw_ball.ball_motor_rel_pos))?;
    };
    robo.steering.off()?;
    Ok(())
}

fn mqtt(addr: String) {
    std::thread::spawn(move || {
        let mqtt_options = MqttOptions::new("HerrBert", addr, 1883);
        let (mut client, mut connection) = Client::new(mqtt_options, 10);
        println!("mqtt connected");
        std::thread::spawn(move || {
            let l_motor = LargeMotor::get(MotorPort::OutA).unwrap();
            let r_motor = LargeMotor::get(MotorPort::OutD).unwrap();
            let b_motor = LargeMotor::get(MotorPort::OutB).unwrap();
            let left_sensor = LightSensor::get(SensorPort::In1).unwrap();
            let middle_sensor = ColorSensor::get(SensorPort::In3).unwrap();
            let right_sensor = LightSensor::get(SensorPort::In4).unwrap();
            let us_sensor = UltrasonicSensor::get(SensorPort::In2).unwrap();
            let psu = PowerSupply::new().unwrap();
            loop {
                client
                    .publish(
                        "robo/line/left",
                        QoS::AtMostOnce,
                        false,
                        left_sensor.get_value0().unwrap().to_string(),
                    )
                    .unwrap();
                client
                    .publish(
                        "robo/line/middle",
                        QoS::AtMostOnce,
                        false,
                        middle_sensor.get_value0().unwrap().to_string(),
                    )
                    .unwrap();
                client
                    .publish(
                        "robo/line/right",
                        QoS::AtMostOnce,
                        false,
                        right_sensor.get_value0().unwrap().to_string(),
                    )
                    .unwrap();
                client
                    .publish(
                        "robo/us",
                        QoS::AtMostOnce,
                        false,
                        us_sensor.get_distance_centimeters().unwrap().to_string(),
                    )
                    .unwrap();
                client
                    .publish(
                        "robo/l_motor/set_speed",
                        QoS::AtMostOnce,
                        false,
                        l_motor.get_speed_sp().unwrap().to_string(),
                    )
                    .unwrap();
                client
                    .publish(
                        "robo/r_motor/set_speed",
                        QoS::AtMostOnce,
                        false,
                        r_motor.get_speed_sp().unwrap().to_string(),
                    )
                    .unwrap();
                client
                    .publish(
                        "robo/b_motor/set_speed",
                        QoS::AtMostOnce,
                        false,
                        b_motor.get_speed_sp().unwrap().to_string(),
                    )
                    .unwrap();
                client
                    .publish(
                        "robo/l_motor/speed",
                        QoS::AtMostOnce,
                        false,
                        l_motor.get_speed().unwrap().to_string(),
                    )
                    .unwrap();
                client
                    .publish(
                        "robo/r_motor/speed",
                        QoS::AtMostOnce,
                        false,
                        r_motor.get_speed().unwrap().to_string(),
                    )
                    .unwrap();
                client
                    .publish(
                        "robo/b_motor/speed",
                        QoS::AtMostOnce,
                        false,
                        b_motor.get_speed().unwrap().to_string(),
                    )
                    .unwrap();
                client
                    .publish(
                        "robo/l_motor/states",
                        QoS::AtMostOnce,
                        false,
                        format!("{:?}", l_motor.get_state().unwrap()),
                    )
                    .unwrap();
                client
                    .publish(
                        "robo/r_motor/states",
                        QoS::AtMostOnce,
                        false,
                        format!("{:?}", r_motor.get_state().unwrap()),
                    )
                    .unwrap();
                client
                    .publish(
                        "robo/b_motor/states",
                        QoS::AtMostOnce,
                        false,
                        format!("{:?}", b_motor.get_state().unwrap()),
                    )
                    .unwrap();
                client
                    .publish(
                        "robo/l_motor/position",
                        QoS::AtMostOnce,
                        false,
                        l_motor.get_position().unwrap().to_string(),
                    )
                    .unwrap();
                client
                    .publish(
                        "robo/r_motor/position",
                        QoS::AtMostOnce,
                        false,
                        r_motor.get_position().unwrap().to_string(),
                    )
                    .unwrap();
                client
                    .publish(
                        "robo/b_motor/position",
                        QoS::AtMostOnce,
                        false,
                        b_motor.get_position().unwrap().to_string(),
                    )
                    .unwrap();
                client.publish("robo/psu/amps", QoS::AtMostOnce, false, (psu.get_current_now().unwrap() as f32/1000000.0).to_string()).unwrap();
                client.publish("robo/psu/volts", QoS::AtMostOnce, false, (psu.get_voltage_now().unwrap()as f32/1000000.0).to_string()).unwrap();
                std::thread::sleep(std::time::Duration::from_millis(75));
            }
        });
        for (_i, _notification) in connection.iter().enumerate() {}
    });
}

fn main() -> Ev3Result<()> {
    let started = std::time::Instant::now();
    let args: Args = Args::parse();

    let settings: settings::Settings = settings::Settings::new(
        args.override_,
        args.settings.unwrap_or(std::path::PathBuf::from("")),
    )
    .unwrap();

    let (left_sensor_threshold, middle_sensor_threshold, right_sensor_threshold) = if args.calibrate
    {
        calibrate_sensors()?
    } else {
        (
            settings.sensors.left_threshold,
            settings.sensors.middle_threshold,
            settings.sensors.right_threshold,
        )
    };

    let mut robo = Robot::new(
        MoveSteering::new(
            LargeMotor::get(MotorPort::OutA)?,
            LargeMotor::get(MotorPort::OutD)?,
            settings.steering.stop_action,
        )?,
        LineSensor::new(
            LineSensorType::Light(LightSensor::get(SensorPort::In1)?),
            left_sensor_threshold,
            false,
        )?,
        LineSensor::new(
            LineSensorType::Light(LightSensor::get(SensorPort::In4)?),
            right_sensor_threshold,
            false,
        )?,
        LineSensor::new(
            LineSensorType::Color(ColorSensor::get(SensorPort::In3)?),
            middle_sensor_threshold,
            true,
        )?,
        UltrasonicSensor::get(SensorPort::In2)?,
        LargeMotor::get(MotorPort::OutB)?,
        settings,
    );

    let turn = Task::new(
        ("turn").to_string(),
        Box::new(act_turn),
        Box::new(cond_dist),
    );
    let catch_ball = Task::new(
        ("catch ball").to_string(),
        Box::new(act_wait_for_ball),
        Box::new(cond_dist),
    );
    let push_block = Task::new(
        ("push block").to_string(),
        Box::new(act_push_block),
        Box::new(cond_lines),
    );
    let throw_ball = Task::new(
        ("throw ball").to_string(),
        Box::new(act_throw_ball),
        Box::new(cond_dist),
    );

    let stop = Task::new(
        ("stop").to_string(),
        Box::new(act_stop),
        Box::new(cond_dist),
    );

    let party = Task::new(
        ("party").to_string(),
        Box::new(act_party),
        Box::new(cond_party),
    );

    ctrlc::set_handler(move || {
        std::thread::sleep(std::time::Duration::from_millis(100));
        let l_motor = LargeMotor::get(MotorPort::OutA).unwrap();
        let r_motor = LargeMotor::get(MotorPort::OutD).unwrap();
        let b_motor = LargeMotor::get(MotorPort::OutB).unwrap();
        l_motor.stop().unwrap();
        r_motor.stop().unwrap();
        b_motor.stop().unwrap();
        l_motor.wait_until_not_moving(None);
        r_motor.wait_until_not_moving(None);
        b_motor.wait_until_not_moving(None);
        std::thread::sleep(std::time::Duration::from_millis(1000));
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    if args.mqtt {
        mqtt(args.mqtt_address.unwrap_or_default())
    }

    if args.all {
        robo.follow_line_loop(Some(turn))?;
        robo.follow_line_loop(Some(catch_ball))?;
        robo.follow_line_loop(Some(push_block))?;
        robo.follow_line_loop(Some(throw_ball))?;
        robo.follow_line_loop(Some(party))?;
    } else if args.none {
        robo.follow_line_loop(None)?;
    } else if args.stop {
        robo.steering.off()?;
        robo.steering.wait_until_not_moving()?;
    } else {
        if args.turn {
            robo.follow_line_loop(Some(turn))?;
        }
        if args.wait_for_ball {
            robo.follow_line_loop(Some(catch_ball))?;
        }
        if args.push_block {
            robo.follow_line_loop(Some(push_block))?;
        }
        if args.throw_ball {
            robo.follow_line_loop(Some(throw_ball))?;
        }
        if args.stop_dist {
            robo.follow_line_loop(Some(stop))?;
        }
        if args.party {
            robo.follow_line_loop(Some(party))?;
        }
    };
    println!("Time elapsed: {:?}", started.elapsed().as_secs_f32());
    Ok(())
}
