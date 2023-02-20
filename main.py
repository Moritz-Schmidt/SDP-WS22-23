#!/usr/bin/env python3

from ev3dev2.motor import (
    LargeMotor,
    OUTPUT_A,
    OUTPUT_B,
    OUTPUT_C,
    OUTPUT_D,
    SpeedPercent,
    MoveTank,
    MoveSteering,
)
from ev3dev2.sensor.lego import UltrasonicSensor, LightSensor, Sensor, ColorSensor
from ev3dev2.sensor import INPUT_1, INPUT_2, INPUT_3, INPUT_4
import time
from typing import List, Tuple, Callable, Any, Optional
from ev3dev2.sound import Sound
import argparse
parser = argparse.ArgumentParser("Robot!")
parser.add_argument("--wenden", action="store_true")
parser.add_argument("--ball", action="store_true")
parser.add_argument("--block", action="store_true")
parser.add_argument("--ballthrow", action="store_true")
parser.add_argument("--all", action="store_true")
parser.add_argument("--none", action="store_true")
parser.add_argument("--speed", type=int, default=100)
parser.add_argument("--steer", type=int, default=70)


class Task:
    def __init__(self, name: str, condition: Callable, action: Callable):
        self.name = name
        self.condition = condition
        self.action = action

    def __call__(self):
        return self.action()

    def __bool__(self):
        return self.condition()


class LineSensor:
    sensor_type = "ls"
    threshold = 30
    on_line_hist = []
    has_history = False

    def __init__(
        self,
        port: str,
        threshold: int = 45,
        sensor_type: str = "ls",
        has_history: bool = False,
    ):
        self.threshold = threshold
        self.sensor_type = sensor_type
        if sensor_type == "ls":
            self.sensor = LightSensor(port)
        elif sensor_type == "cs":
            self.sensor = ColorSensor(port)
        else:
            raise ValueError("sensor_type must be 'ls' or 'cs'")
        if has_history:
            self.on_line_hist = [(False, 0)]
            self.has_history = True

    def append_line_hist(self, on_line: bool):
        if self.has_history:
            if self.on_line_hist[-1][0] != on_line:
                self.on_line_hist.append((on_line, time.time()))  # type: ignore
            [
                self.on_line_hist.remove(el)
                for el in self.on_line_hist
                if time.time() - el[1] > 1 and len(self.on_line_hist) > 1
            ]

    @property
    def on_line(self):
        on_line = self.reflected_light_intensity <= self.threshold
        self.append_line_hist(on_line)
        return on_line

    @property
    def reflected_light_intensity(self):
        return self.sensor.reflected_light_intensity

    @property
    def off_line(self):
        return not self.on_line


left_motor_port = OUTPUT_A
right_motor_port = OUTPUT_D
left_sensor_port = INPUT_1
middle_sensor_port = INPUT_3
right_sensor_port = INPUT_4

max_steer = 70
speed = 100

left_motor = LargeMotor(left_motor_port)
right_motor = LargeMotor(right_motor_port)
left_sensor = LineSensor(left_sensor_port, threshold=37)
middle_sensor = LineSensor(middle_sensor_port, sensor_type="cs", has_history=True, threshold=25)
right_sensor = LineSensor(right_sensor_port, threshold=40)
us_sensor = UltrasonicSensor(INPUT_2)
ball_motor = LargeMotor(OUTPUT_B)
robo = MoveSteering(left_motor_port, right_motor_port)
spkr = Sound()
motor_hist = []


def act_wenden():
    robo.on_for_rotations(100, speed, 1.1)
    robo.on(100, speed)
    while middle_sensor.off_line:
        pass
    robo.off()
    return True


def cond_dist():
    return us_sensor.distance_centimeters < 20


def act_wait_for_ball():
    while us_sensor.distance_centimeters > 5:
        follow_line()
    robo.off()
    dist = [us_sensor.distance_centimeters]
    start = time.perf_counter()
    while True:
        dist.append(us_sensor.distance_centimeters)
        if len(dist) > 5:
            del dist[0]
            if sum(dist) / len(dist) > 30:
                return True
        if time.perf_counter() - start > 10:
            return False


def cond_lines():
    return len(middle_sensor.on_line_hist) > 5


def act_push_block():
    # motor_hist.clear()
    # motor_hist.append((left_motor.position, right_motor.position))
    robo.on_for_rotations(70, 100, 1)
    # motor_hist.append((left_motor.position, right_motor.position))
    robo.on(100, 20)
    while not middle_sensor.on_line:
        pass
    robo.off()
    # motor_hist.append((left_motor.position, right_motor.position))
    while not right_sensor.on_line == left_sensor.on_line == middle_sensor.on_line:
        follow_line()
    #robo.on_for_rotations(0, -100, 0.2)
    act_wenden()
    while not right_sensor.on_line == left_sensor.on_line == middle_sensor.on_line:
        follow_line()
    left_motor.on_for_seconds(100, 2, block=False)
    right_motor.on_for_seconds(15, 2, block=False)
    time.sleep(2)
    robo.on(100, 20)
    while not middle_sensor.on_line:
        pass
    robo.off()
    return True

def act_throw_ball():
    while us_sensor.distance_centimeters > 5:
        follow_line()
    robo.on(0, 50)
    robo.wait_until('stalled', timeout=1000)
    robo.off()

    pos = ball_motor.position
    ball_motor.on_for_degrees(100, 80)
    time.sleep(0.2)
    ball_motor.on_to_position(100, pos)


def motor_history(func: Callable):
    def wrapper(hist: bool = False, *args, **kwargs):
        if hist:
            global motor_hist
            motor_hist.append((left_motor.position, right_motor.position))
        return func(*args, **kwargs)

    return wrapper


def hist_back():
    hist = motor_hist[::-1]
    for (l, r) in hist:
        left_motor.on_to_position(100, l, block=False)
        right_motor.on_to_position(100, r, block=False)
        while left_motor.is_running or right_motor.is_running:
            pass
    motor_hist.clear()
    robo.off()


# @motor_history
def turn_right_until_line(hist: bool = False):
    robo.on(max_steer, speed)
    while right_sensor.on_line:
        pass
    robo.off()


# @motor_history
def turn_left_until_line(hist: bool = False):
    robo.on(-max_steer, speed)
    while left_sensor.on_line:
        pass
    robo.off()


# @motor_history
def forward(hist: bool = False):
    robo.on(0, speed)


def follow_line_loop(task: Optional[Task] = None):
    while True:
        if task:
            return task()
        follow_line()


def follow_line(hist: bool = False):
    if left_sensor.off_line and middle_sensor.on_line and right_sensor.off_line:
        forward(hist)
    elif left_sensor.off_line and middle_sensor.on_line and right_sensor.on_line:
        turn_right_until_line(hist)
    elif left_sensor.on_line and middle_sensor.on_line and right_sensor.off_line:
        turn_left_until_line(hist)
    elif left_sensor.off_line and middle_sensor.off_line and right_sensor.off_line:
        forward(hist)
    elif left_sensor.on_line and middle_sensor.off_line and right_sensor.off_line:
        turn_left_until_line(hist)
    elif left_sensor.off_line and middle_sensor.off_line and right_sensor.on_line:
        turn_right_until_line(hist)
    else:
        forward(hist)

def follow_line_reverse():
    # 100 left
    # 0 forward
    # -100 right
    if left_sensor.off_line and middle_sensor.on_line and right_sensor.off_line:
        robo.on(0, -speed)
    elif left_sensor.off_line and middle_sensor.on_line and right_sensor.on_line:
        robo.on(max_steer, -speed)
        while right_sensor.on_line:
            pass
        robo.off()
    elif left_sensor.on_line and middle_sensor.on_line and right_sensor.off_line:
        robo.on(-max_steer, -speed)
        while left_sensor.on_line:
            pass
        robo.off()
    elif left_sensor.off_line and middle_sensor.off_line and right_sensor.off_line:
        robo.on(0, -speed)
    elif left_sensor.on_line and middle_sensor.off_line and right_sensor.off_line:
        robo.on(-max_steer, -speed)
        while left_sensor.on_line:
            pass
        robo.off()
    elif left_sensor.off_line and middle_sensor.off_line and right_sensor.on_line:
        robo.on(max_steer, -speed)
        while right_sensor.on_line:
            pass
        robo.off()
    else:
        robo.on(0, -speed)


def run(tasks: List[Task]):
    for task in tasks:
        follow_line_loop(task)
    robo.off()


def configure_sensors():
    lsv_l = []
    msv_l = []
    rsv_l = []
    print("put sensors on line")
    input("press enter to continue")
    for _ in range(20):
        lsv_l.append(left_sensor.reflected_light_intensity)
        msv_l.append(middle_sensor.reflected_light_intensity)
        rsv_l.append(right_sensor.reflected_light_intensity)
        time.sleep(0.1)
    lsv_w = []
    msv_w = []
    rsv_w = []
    print("put sensors on white")
    input("press enter to continue")
    for _ in range(20):
        lsv_w.append(left_sensor.reflected_light_intensity)
        msv_w.append(middle_sensor.reflected_light_intensity)
        rsv_w.append(right_sensor.reflected_light_intensity)
        time.sleep(0.1)
    left_sensor.threshold = int(
        abs(sum(lsv_w) / len(lsv_w) + sum(lsv_l) / len(lsv_l)) / 2
    )
    middle_sensor.threshold = int(
        abs(sum(msv_w) / len(msv_w) + sum(msv_l) / len(msv_l)) / 2
    )
    right_sensor.threshold = int(
        abs(sum(rsv_w) / len(rsv_w) + sum(rsv_l) / len(rsv_l)) / 2
    )
    print("thresholds set")
    print("left: ", left_sensor.threshold)
    print("middle: ", middle_sensor.threshold)
    print("right: ", right_sensor.threshold)


wenden = Task("wenden", cond_dist, act_wenden)
wait_for_ball = Task("Ball Fangen", cond_dist, act_wait_for_ball)
push_block = Task("push block", cond_lines, act_push_block)
throw_ball = Task("throw ball", cond_dist, act_throw_ball)
tasks = [wenden, wait_for_ball, throw_ball]

def set_tasks():
    global tasks, speed, max_steer
    args = parser.parse_args()
    if args.all:
        return tasks
    if args.none:
        return []
    speed = args.speed
    max_steer = args.steer
    tasks = []
    if args.wenden:
        tasks.append(wenden)
    if args.ball:
        tasks.append(wait_for_ball)
    if args.block:
        tasks.append(push_block)
    if args.ballthrow:
        tasks.append(throw_ball)
    return tasks

if __name__ == "__main__":
    tasks = set_tasks()
    print([t.name for t in tasks])
    while True:
        input("press enter to start")
        try:
            run(tasks)
            while True:
                follow_line()
        except:
            robo.off()
