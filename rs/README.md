# Rust Programm zur steuerung des Roboters

## Installation
### Voraussetzungen
- Rust und Cargo (https://www.rust-lang.org/tools/install)
- armv5te-unknown-linux-musleabi toolchain (`rustup target add armv5te-unknown-linux-musleabi`)
- Optional: MQTT Broker (https://mosquitto.org/download/) mit Websocket Unterstützung

### Installation
(funktioniert wahrscheinlich nur auf Linux)
```sh
cargo build --release
```
`target/armv5te-unknown-linux-musleabi/release/` auf den EV3 kopieren und ausführen.

Mehr Infos zum compilieren für den EV3 gibt es hier:

https://github.com/pixix4/ev3dev-lang-rust

## Konfiguration
`sdp2023 --help` zeigt alle verfügbaren Optionen an.
`settings/default.yaml` ist die Standardkonfiguration.


