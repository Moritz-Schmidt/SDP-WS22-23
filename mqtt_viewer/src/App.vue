<script setup lang="ts">
import * as mqtt from "mqtt";
import { Chart } from "chart.js";
import "chart.js/auto";
import "chartjs-adapter-luxon";
import annotationPlugin from "chartjs-plugin-annotation";
import ChartStreaming from "chartjs-plugin-streaming";
import { ref, onMounted, reactive } from "vue";
import { nextTick } from "process";

Chart.register(ChartStreaming, annotationPlugin);

const possibleStates = ["running", "ramping", "holding", "overloaded", "stalled" ]

const leftsensor = ref<HTMLCanvasElement | null>(null);
const rightsensor = ref<HTMLCanvasElement | null>(null);
const middlesensor = ref<HTMLCanvasElement | null>(null);
const leftmotor = ref<HTMLCanvasElement | null>(null);
const rightmotor = ref<HTMLCanvasElement | null>(null);
const us = ref<HTMLCanvasElement | null>(null);

let leftsensor_chart: Chart;
let rightsensor_chart: Chart;
let middlesensor_chart: Chart;
let leftmotor_chart: Chart;
let rightmotor_chart: Chart;
let us_chart: Chart;



let robo = reactive({
  thresholds: {
    left: 400,
    middle: 10,
    right: 400,
  },
  sensors: {
    left: 0,
    middle: 0,
    right: 0,
  },
  us: 0,
  l_motor: {
    set_speed: 0,
    speed: 0,
    states: [""],
  },
  r_motor: {
    set_speed: 0,
    speed: 0,
    states: [""],
  },
  b_motor: {
    set_speed: 0,
    speed: 0,
    states: [""],
  },
});

let sensor_chart_scales = reactive({
  x: {
    type: "realtime",
    realtime: {
      duration: 30000,
      refresh: 100,
    },
  },
});
onMounted(() => {
  if (leftsensor.value) {
    leftsensor_chart = new Chart(leftsensor.value, {
      type: "line",
      options: {
        animation: false,
        scales: sensor_chart_scales,
        plugins: {
          annotation: {
            annotations: {
              threshold: {
                type: "line",
                yMin: robo.thresholds.left,
                yMax: robo.thresholds.left,
                borderColor: "rgb(255, 50, 50)",
                borderWidth: 2,
              },
            },
          },
        },
      },
      data: {
        datasets: [
          {
            label: "Left Sensor",
            data: [],
            cubicInterpolationMode: "monotone",
          },
        ],
      },
    });
  }

  if (middlesensor.value) {
    middlesensor_chart = new Chart(middlesensor.value, {
      type: "line",
      options: {
        animation: false,
        scales: sensor_chart_scales,
        plugins: {
          annotation: {
            annotations: {
              threshold: {
                type: "line",
                yMin: robo.thresholds.middle,
                yMax: robo.thresholds.middle,
                borderColor: "rgb(255, 50, 50)",
                borderWidth: 2,
              },
            },
          },
        },
      },
      data: {
        datasets: [
          {
            label: "Middle Sensor",
            data: [],
            cubicInterpolationMode: "monotone",
          },
        ],
      },
    });
  }

  if (rightsensor.value) {
    rightsensor_chart = new Chart(rightsensor.value, {
      type: "line",
      options: {
        animation: false,
        scales: sensor_chart_scales,
        plugins: {
          annotation: {
            annotations: {
              threshold: {
                type: "line",
                yMin: robo.thresholds.right,
                yMax: robo.thresholds.right,
                borderColor: "rgb(255, 50, 50)",
                borderWidth: 2,
              },
            },
          },
        },
      },
      data: {
        datasets: [
          {
            label: "Right Sensor",
            data: [],
            cubicInterpolationMode: "monotone",
          },
        ],
      },
    });
  }

  if (leftmotor.value) {
    leftmotor_chart = new Chart(leftmotor.value, {
      type: "line",
      options: {
        animation: false,
        aspectRatio: 1.5,
        spanGaps: 1000*5,
        scales: {
          x: {
            type: "realtime",
            realtime: {
              duration: 30000,
              refresh: 100,
            },
          },
          ySet_speed: {
            type: "linear",
            position: "left",
            stack: "left",
            stackWeight: 4,
            border: {
              color: "#36A2EB"
            }
          },
          yState: {
            type: "category",
            labels: ["running", "ramping", "holding", "overloaded", "stalled" ],
            min: 0,
            max: 5,
            position: "left",
            offset: true,
            stack: "left",
            stackWeight: 2,
            border: {
              color: "#40E727"
            },
            ticks: {
              autoSkip: false
            },
          },
          yPlaceholder: {
            type: "category",
            labels: ["running", "ramping", "holding", "overloaded", "stalled" ],
            position: "right",
            stack: "right",
            offset: true,
            stackWeight: 2,
            ticks: {
              autoSkip: false
            },
            grid: {
              drawOnChartArea: false, // only want the grid lines for one axis to show up
            },
            border: {
              color: "#40E727"
            }
          },
          yActual_speed: {
            type: "linear",
            position: "right",
            stack: "right",
            stackWeight: 4,
            grid: {
              drawOnChartArea: false, // only want the grid lines for one axis to show up
            },
            border: {
              color: "#FF6384"}
          },
        },
        plugins: {
          legend: {
                  display: true,
                  labels: {
                      fontSize: 16, //point style's size is based on font style not boxed width.
                      usePointStyle: true,

                  }
              }
          }
      },
      data: {
        datasets: [
          {
            label: "Right Motor set speed",
            data: [],
            cubicInterpolationMode: "monotone",
            yAxisID: "ySet_speed",
            borderColor: "#36A2EB",
            backgroundColor: "#97CDF2"
          },
          {
            label: "Right Motor actual speed",
            data: [],
            cubicInterpolationMode: "monotone",
            yAxisID: "yActual_speed",
            borderColor: "#FF6384",
            backgroundColor: "#FCADBE"

          },
          {
            label: "running",
            data: [],
            borderColor: "rgba(0, 255, 0, 1)",
            backgroundColor: "rgba(0, 255, 0, 0.4)",
            borderWidth: 20,
            pointBorderWidth: 1,
            yAxisID: "yState"
          },
          {
            label: "ramping",
            data: [],
            borderColor: "rgba(255, 255, 0, 1)",
            backgroundColor: "rgba(255, 255, 0, 0.4)",
            borderWidth: 20,
            pointBorderWidth: 1,
            yAxisID: "yState"
          },
          {
            label: "holding",
            data: [],
            borderColor: "rgba(0, 150, 255, 1)",
            backgroundColor: "rgba(0, 150, 155, 0.4)",
            borderWidth: 20,
            pointBorderWidth: 1,
            yAxisID: "yState"
          },
          {
            label: "overloaded",
            data: [],
            borderColor: "rgba(255, 150, 0, 1)",
            backgroundColor: "rgba(255, 150, 0, 0.4)",
            borderWidth: 20,
            pointBorderWidth: 1,
            yAxisID: "yState"
          },
          {
            label: "stalled",
            data: [],
            borderColor: "rgba(255, 0, 0, 1)",
            backgroundColor: "rgba(255, 0, 0, 0.4)",
            borderWidth: 20,
            pointBorderWidth: 1,
            yAxisID: "yState"
          },
        ],
      },
    });
  }

  if (rightmotor.value) {
    rightmotor_chart = new Chart(rightmotor.value, {
      type: "line",
      options: {
        animation: false,
        aspectRatio: 1.5,
        spanGaps: 1000*5,
        scales: {
          x: {
            type: "realtime",
            realtime: {
              duration: 30000,
              refresh: 100,
            },
          },
          ySet_speed: {
            type: "linear",
            position: "left",
            stack: "left",
            stackWeight: 4,
            border: {
              color: "#36A2EB"
            }
          },
          yState: {
            type: "category",
            labels: ["running", "ramping", "holding", "overloaded", "stalled" ],
            min: 0,
            max: 5,
            position: "left",
            offset: true,
            stack: "left",
            stackWeight: 2,
            border: {
              color: "#40E727"
            },
            ticks: {
              autoSkip: false
            },
          },
          yPlaceholder: {
            type: "category",
            labels: ["running", "ramping", "holding", "overloaded", "stalled" ],
            position: "right",
            stack: "right",
            offset: true,
            stackWeight: 2,
            ticks: {
              autoSkip: false
            },
            grid: {
              drawOnChartArea: false, // only want the grid lines for one axis to show up
            },
            border: {
              color: "#40E727"
            }
          },
          yActual_speed: {
            type: "linear",
            position: "right",
            stack: "right",
            stackWeight: 4,
            grid: {
              drawOnChartArea: false, // only want the grid lines for one axis to show up
            },
            border: {
              color: "#FF6384"}
          },
        },
        plugins: {
          legend: {
                  display: true,
                  labels: {
                      fontSize: 16, //point style's size is based on font style not boxed width.
                      usePointStyle: true,

                  }
              }
          }
      },
      data: {
        datasets: [
          {
            label: "Right Motor set speed",
            data: [],
            cubicInterpolationMode: "monotone",
            yAxisID: "ySet_speed",
            borderColor: "#36A2EB",
            backgroundColor: "#97CDF2"
          },
          {
            label: "Right Motor actual speed",
            data: [],
            cubicInterpolationMode: "monotone",
            yAxisID: "yActual_speed",
            borderColor: "#FF6384",
            backgroundColor: "#FCADBE"

          },
          {
            label: "running",
            data: [],
            borderColor: "rgba(0, 255, 0, 1)",
            backgroundColor: "rgba(0, 255, 0, 0.4)",
            borderWidth: 20,
            pointBorderWidth: 1,
            yAxisID: "yState"
          },
          {
            label: "ramping",
            data: [],
            borderColor: "rgba(255, 255, 0, 1)",
            backgroundColor: "rgba(255, 255, 0, 0.4)",
            borderWidth: 20,
            pointBorderWidth: 1,
            yAxisID: "yState"
          },
          {
            label: "holding",
            data: [],
            borderColor: "rgba(0, 150, 255, 1)",
            backgroundColor: "rgba(0, 150, 155, 0.4)",
            borderWidth: 20,
            pointBorderWidth: 1,
            yAxisID: "yState"
          },
          {
            label: "overloaded",
            data: [],
            borderColor: "rgba(255, 150, 0, 1)",
            backgroundColor: "rgba(255, 150, 0, 0.4)",
            borderWidth: 20,
            pointBorderWidth: 1,
            yAxisID: "yState"
          },
          {
            label: "stalled",
            data: [],
            borderColor: "rgba(255, 0, 0, 1)",
            backgroundColor: "rgba(255, 0, 0, 0.4)",
            borderWidth: 20,
            pointBorderWidth: 1,
            yAxisID: "yState"
          },
        ],
      },
    });
  }

  if (us.value) {
    us_chart = new Chart(us.value, {
      type: "line",
      options: {
        animation: false,
        scales:   {
          x: {
            type: "realtime",
            realtime: {
              duration: 30000,
              refresh: 100,
            },
          },
          y: {
            type: "logarithmic",
          }
        },
        plugins: {
          annotation: {
            annotations: {
              threshold: {
                type: "line",
                yMin: 20,
                yMax: 20,
                borderColor: "rgb(255, 50, 50)",
                borderWidth: 2,
              },
            },
          },
        },
      },
      data: {
        datasets: [
          {
            label: "US Sensor",
            data: [],
            cubicInterpolationMode: "monotone",
          },
        ],
      },
    });
  }

  const clientId = "mqttjs_" + Math.random().toString(16).substring(2, 8);

  const host = "ws://127.0.0.1:1312/";

  const topics = [
    "robo/line/left",
    "robo/line/middle",
    "robo/line/right",
    "robo/us",
    "robo/l_motor/set_speed",
    "robo/r_motor/set_speed",
    "robo/b_motor/set_speed",
    "robo/l_motor/speed",
    "robo/r_motor/speed",
    "robo/b_motor/speed",
    "robo/l_motor/states",
    "robo/r_motor/states",
    "robo/b_motor/states",
  ];
  console.log("Connecting mqtt client");
  const client = mqtt.connect(host, {
    clientId,
    clean: false,
    reconnectPeriod: 1000,
    connectTimeout: 30 * 1000,
  });

  client.on("error", (err) => {
    console.log("Connection error: ", err);
    client.end();
  });

  client.on("reconnect", () => {
    console.log("Reconnecting...");
  });

  client.on("connect", () => {
    console.log("Client connected:" + clientId);
    client.subscribe(topics, { qos: 0 }, (err, granted) => {
      if (!err) {
        console.log("subscribe success");
      }
    });
  });

  client.on("message", (topic, message, packet) => {
    let states: string[]
    switch (topic) {
      case "robo/line/left":
        robo.sensors.left = parseInt(message.toString());
        leftsensor_chart.data.datasets[0].data.push({
          x: Date.now(),
          y: parseInt(message.toString()),
        });
        break;
      case "robo/line/middle":
        robo.sensors.middle = parseInt(message.toString());
        middlesensor_chart.data.datasets[0].data.push({
          x: Date.now(),
          y: parseInt(message.toString()),
        });
        break;
      case "robo/line/right":
        robo.sensors.right = parseInt(message.toString());
        rightsensor_chart.data.datasets[0].data.push({
          x: Date.now(),
          y: parseInt(message.toString()),
        });
        break;
      case "robo/us":
        robo.us = parseInt(message.toString());
        us_chart.data.datasets[0].data.push({
          x: Date.now(),
          y: parseInt(message.toString()),
        });
        break;
      case "robo/l_motor/set_speed":
        robo.l_motor.set_speed = parseInt(message.toString());
        leftmotor_chart.data.datasets[0].data.push({
          x: Date.now(),
          y: parseInt(message.toString()),
        });
        break;
      case "robo/r_motor/set_speed":
        robo.r_motor.set_speed = parseInt(message.toString());
        rightmotor_chart.data.datasets[0].data.push({
          x: Date.now(),
          y: parseInt(message.toString()),
        });
        break;
      case "robo/b_motor/set_speed":
        robo.b_motor.set_speed = parseInt(message.toString());
        break;
      case "robo/l_motor/speed":
        robo.l_motor.speed = parseInt(message.toString());
        leftmotor_chart.data.datasets[1].data.push({
          x: Date.now(),
          y: parseInt(message.toString()),
        });
        break;
      case "robo/r_motor/speed":
        robo.r_motor.speed = parseInt(message.toString());
        rightmotor_chart.data.datasets[1].data.push({
          x: Date.now(),
          y: parseInt(message.toString()),
        });
        break;
      case "robo/b_motor/speed":
        robo.b_motor.speed = parseInt(message.toString());
        break;
      case "robo/l_motor/states":
        states = JSON.parse(message.toString());
        robo.l_motor.states = states;
        possibleStates.forEach((state: String) => {
          leftmotor_chart.data.datasets.find((dataset) => {
            return dataset.label == state;
          })?.data.push({
            x: Date.now(),
            y: states.includes(state) ? state : null,
          });
        });
        break;
      case "robo/r_motor/states":
        states = JSON.parse(message.toString());
        robo.r_motor.states = states;
        possibleStates.forEach((state: String) => {
          rightmotor_chart.data.datasets.find((dataset) => {
            return dataset.label == state;
          })?.data.push({
            x: Date.now(),
            y: states.includes(state) ? state : null,
          });
        })
        break;
      case "robo/b_motor/states":
        robo.b_motor.states = JSON.parse(message.toString());
    }
  });
});

function updateThreshold(sensor: String, val: Event) {
  console.log(val);
  console.log(rightsensor_chart);
  switch (sensor) {
    case "right":
      rightsensor_chart.config.options.plugins.annotation.annotations.threshold.yMin =
        val.target.value;
      rightsensor_chart.config.options.plugins.annotation.annotations.threshold.yMax =
        val.target.value;
      break;
    case "middle":
      middlesensor_chart.config.options.plugins.annotation.annotations.threshold.yMin =
        val.target.value;
      middlesensor_chart.config.options.plugins.annotation.annotations.threshold.yMax =
        val.target.value;
      break;
    case "left":
      leftsensor_chart.config.options.plugins.annotation.annotations.threshold.yMin =
        val.target.value;
      leftsensor_chart.config.options.plugins.annotation.annotations.threshold.yMax =
        val.target.value;
      break;
  }
  nextTick(() => rightsensor_chart.update());
}

function updateDuration(val: Event) {
  rightsensor_chart.config.options.scales.x.realtime.duration =
    val.target.value;
  middlesensor_chart.config.options.scales.x.realtime.duration =
    val.target.value;
  leftsensor_chart.config.options.scales.x.realtime.duration = val.target.value;
  leftmotor_chart.config.options.scales.x.realtime.duration = val.target.value;
  rightmotor_chart.config.options.scales.x.realtime.duration = val.target.value;
}
</script>

<template>
  <div>
    <div class="wrapper">
      <!--<p v-for="key in Object.keys(robo)" :key="key">
        {{ key }}: {{ robo[key] }}
      </p>-->
      <div class="threshold-inputs">
        <div class="threshold-input">
          <label for="left-threshold">Left: </label>
          <input
            type="number"
            id="left-threshold"
            v-model="robo.thresholds.left"
            @input="(val) => updateThreshold('left', val)"
          />
        </div>
        <div class="threshold-input">
          <label for="middle-threshold">Middle: </label>
          <input
            type="number"
            id="middle-threshold"
            v-model="robo.thresholds.middle"
            @input="(val) => updateThreshold('middle', val)"
          />
        </div>
        <div class="threshold-input">
          <label for="right-threshold">Right: </label>
          <input
            type="number"
            id="right-threshold"
            v-model="robo.thresholds.right"
            @input="(val) => updateThreshold('right', val)"
          />
        </div>
        <div class="threshold-input">
          <label for="duration">Duration: </label>
          <input
            type="number"
            id="duration"
            v-model="sensor_chart_scales.x.realtime.duration"
            @input="(val) => updateDuration(val)"
          />
        </div>
      </div>
      <div class="sensor-data">
        <table>
          <thead>
            <tr>
              <th>Left</th>
              <th>Middle</th>
              <th>Right</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td>
                <div class="center">
                  <div
                    class="dot"
                    :style="{
                      backgroundColor:
                        robo.sensors.left < robo.thresholds.left
                          ? 'black'
                          : 'white',
                    }"
                  ></div>
                  {{ robo.sensors.left }}
                </div>
              </td>
              <td>
                <div class="center">
                  <div
                    class="dot"
                    :style="{
                      backgroundColor:
                        robo.sensors.middle < robo.thresholds.middle
                          ? 'black'
                          : 'white',
                    }"
                  ></div>
                  {{ robo.sensors.middle }}
                </div>
              </td>
              <td>
                <div class="center">
                  <div
                    class="dot"
                    :style="{
                      backgroundColor:
                        robo.sensors.right < robo.thresholds.right
                          ? 'black'
                          : 'white',
                    }"
                  ></div>
                  {{ robo.sensors.right }}
                </div>
              </td>
            </tr>
            <tr>
              <td>
                <div class="center">
                  <div class="chart-wrapper">
                    <canvas ref="leftsensor"></canvas>
                  </div>
                </div>
              </td>
              <td>
                <div class="center">
                  <div class="chart-wrapper">
                    <canvas ref="middlesensor"></canvas>
                  </div>
                </div>
              </td>
              <td>
                <div class="center">
                  <div class="chart-wrapper">
                    <canvas ref="rightsensor"></canvas>
                  </div>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
      <div class="motor-data">
        <table>
          <thead>
            <tr>
              <th>Left Motor</th>
              <th>Right Motor</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td>
                <div class="chart-wrapper">
                  <canvas ref="leftmotor"></canvas>
                </div>
              </td>
              <td>
                <div class="chart-wrapper">
                  <canvas ref="rightmotor"></canvas>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
      <div class="us-data">
        <div class="chart-wrapper">
          <canvas ref="us"></canvas>
        </div>
      </div>

    </div>
  </div>
</template>


<style scoped>
.sensor-data .chart-wrapper {
  width: 20vw;
}

.us-data .chart-wrapper {
  width: 40vw;
  height: 100%;
}
.motor-data .chart-wrapper {
  width: 40vw;
  height: 100%;
}
.wrapper {
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 100vw;
  height: 100%;
}

.sensor-data table {
  width: 80vw;
}
.dot {
  height: 25px;
  width: 25px;
  border-radius: 50%;
  display: inline-block;
  margin: 10px;
}
.center {
  display: flex;
  justify-content: center;
  align-items: center;
}
</style>
