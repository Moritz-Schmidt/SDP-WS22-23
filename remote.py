#!/usr/bin/env python3

from ev3dev2.motor import (
    LargeMotor,
    OUTPUT_A,
    OUTPUT_B,
    OUTPUT_C,
    OUTPUT_D,
    SpeedPercent,
    MoveTank,
)
from ev3dev2.sensor.lego import UltrasonicSensor, ColorSensor, LightSensor
from ev3dev2.sensor import INPUT_1, INPUT_2, INPUT_3, INPUT_4
import time
import curses
from paho.mqtt import client as mqtt_client

# get the curses screen window
screen = curses.initscr()

# turn off input echoing
curses.noecho()

# respond to keys immediately (don't wait for enter)
curses.cbreak()

# map arrow keys to special values
screen.keypad(True)
screen.timeout(50)

lm = LargeMotor(OUTPUT_A)
rm = LargeMotor(OUTPUT_D)
lls = LightSensor(INPUT_1)
rls = LightSensor(INPUT_4)
mcs = ColorSensor(INPUT_3)
us = UltrasonicSensor(INPUT_2)


broker = "192.168.12.1"
port = 1883
topic = "/ev3dev"
client_id = "ev3dev"
client = mqtt_client.Client(client_id)
client.connect(broker, port)
client.loop_start()

start = time.perf_counter()
try:
    while True:
        char = screen.getch()
        lm_position, lm_speed, lm_state = lm.position, lm.speed, lm.state
        rm_position, rm_speed, rm_state = rm.position, rm.speed, rm.state
        lls_reflected_light_intensity = lls.reflected_light_intensity
        rls_reflected_light_intensity = rls.reflected_light_intensity
        mcs_color_name, mcs_rgb, mcs_reflected_light_intensity = (
            mcs.color_name,
            mcs.rgb,
            mcs.reflected_light_intensity,
        )
        us_distance_centimeters = us.distance_centimeters
        screen.addstr(2, 0, str(round(time.perf_counter() - start, 2)))
        screen.addstr(
            3, 0, "Left Light Sensor  |  Middle color Sensor  |  Right Light Sensor"
        )
        screen.addstr(
            4,
            0,
            "{0:<19.2f}|{1:<10} {2:<12}|{3:.2f}".format(
                lls_reflected_light_intensity,
                mcs_color_name,
                str(mcs_rgb),
                rls_reflected_light_intensity,
            ),
        )
        screen.addstr(
            5, 0, "Ultrasonic Sensor: {0:.2f}".format(us_distance_centimeters)
        )
        screen.addstr(
            6, 0, "Left Motor: {0}, {1}, {2}".format(lm_position, lm_speed, lm_state)
        )
        screen.addstr(
            7, 0, "Right Motor: {0}, {1}, {2}".format(rm_position, rm_speed, rm_state)
        )

        client.publish("ev3/left_motor/speed", lm_speed)
        client.publish("ev3/left_motor/position", lm_position)
        client.publish("ev3/left_motor/state", str(lm_state))
        client.publish("ev3/right_motor/speed", rm_speed)
        client.publish("ev3/right_motor/position", rm_position)
        client.publish("ev3/right_motor/state", str(rm_state))
        client.publish("ev3/left_light_sensor", lls_reflected_light_intensity)
        client.publish("ev3/right_light_sensor", rls_reflected_light_intensity)
        client.publish("ev3/middle_color_sensor/color", mcs_color_name)
        client.publish("ev3/middle_color_sensor/rgb", str(mcs_rgb))
        client.publish(
            "ev3/middle_color_sensor/reflected_light_intensity",
            mcs_reflected_light_intensity,
        )
        client.publish("ev3/ultrasonic_sensor", us_distance_centimeters)

        if char == -1:
            lm.off()
            rm.off()
            screen.addstr(0, 0, "no key pressed ")
        elif char == ord("q"):
            break
        elif char == curses.KEY_RIGHT:
            lm.on(-50)
            rm.on(50)
            screen.addstr(0, 0, "right          ")
        elif char == curses.KEY_LEFT:
            lm.on(50)
            rm.on(-50)
            screen.addstr(0, 0, "left           ")
        elif char == curses.KEY_UP:
            lm.on(-50)
            rm.on(-50)
            screen.addstr(0, 0, "forward        ")
        elif char == curses.KEY_DOWN:
            lm.on(50)
            rm.on(50)
            screen.addstr(0, 0, "backward       ")
finally:
    # shut down cleanly
    curses.nocbreak()
    screen.keypad(False)
    curses.echo()
    curses.endwin()
