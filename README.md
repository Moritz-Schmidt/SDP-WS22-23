# SDP 2022/23 Gruppe #126 - 404 group not found
System Design Projekt Wintersemester 2022/23 an der Universität Freiburg.

Gruppenmitglieder: Amelie Dickmann, Hanna Brugger, Luca Fandrey, Moritz Schmidt

## Aufgabenstellung

Jede Gruppe baut und programmiert einen Roboter aus Lego, dieser wird mit einem EV3 angesteuert.
Der Roboter soll einer Linie mit u.a. Lücken, steigungen und Scharfen Kurven folgen und dabei verschiedene Aufgaben ausführen.
Die Aufgaben sind:
1. Wenden wenn 20 cm vor dem Roboter eine Wand ist
2. Vor einer Schranke stehen bleiben und eine Ball aufnehmen
3. Weiterfaren sobald die schranke sich öffnet
4. Muster auf der Linie erkennen, abbiegen und einen Holzklotz wegschieben, anschließend wieder auf die Linie zurückkehren
5. Den Ball am ende der Strecke ablegen


## Unser Code
Mit dem Code in `rs/` konnten wir den Wettbewerb gewinnen.
`mqtt_viewer/` ist ein Vue.js Projekt, welches die Daten des Roboters visualisiert.
`main.py` ist unsere erste Version des Programms in Python.
`remote.py` ist ein Programm mit dem man den Roboter fernsteuern kann, es funktioniert aber nicht wirklich gut.

## Ergebnisse
Ein Video von unserem Roboter beim Wettbewerb gibt es hier: https://www.youtube.com/live/OKjFeBaSAuw?t=23802 (6:36:42 - 6:39:00)
