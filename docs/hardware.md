# Hardware Notes

## Input Circuit

Components:

- Pull-up Resistor -> 5V:
  - Value: 10kΩ
  - Reasoning: Better than pull-up resistor built-in to micro-controller (which is 50kΩ)
- Inline Resistor -> Sensor Signal:
  - Value: 120Ω
  - Reasoning: ???
- Capacitor -> GND:
  - Reasoning: To smooth signal, minimize noise
  - Value: 100nF
- Schmitt Trigger
  - Reasoning: 
  - Value: "74LS14 Hex Schmitt Trigger IC" ???

## Concepts

- [Electrical ("Galvanic") Isolation](https://en.wikipedia.org/wiki/Galvanic_isolation)

## Resources

### optically isolated and debounced 24V switch input

https://forum.allaboutcircuits.com/threads/optically-isolated-and-debounced-24v-switch-input.136328/

### NO (Normally Open) vs NC (Normally Closed)

TODO

### End Stop / Limit Switch Problems

- https://www.reddit.com/r/hobbycnc/comments/qgyxte/limit_switches_nc_circuit_with_cnc_shield/
- https://www.instructables.com/End-Stop-Limit-Switch-Problems/

### Basic Input Cirucit

[Schematic_Quad_Basic_Input_v1p1.pdf](https://raw.githubusercontent.com/bdring/6-Pack_CNC_Controller/main/CNC_IO_Modules/Quad_Input/V1p1/Schematic_Quad_Basic_Input_v1p1.pdf)

### Opto Input Cirucit

[Schematic_CNC_Module_Opto_Schmitt_V1p2.pdf](https://raw.githubusercontent.com/bdring/6-Pack_CNC_Controller/main/CNC_IO_Modules/4x_Opto_Input/V1p2/Schematic_CNC_Module_Opto_Schmitt_V1p2.pdf)

### Schmitt Trigger datasheet

[Jaycar](https://www.jaycar.co.nz/medias/sys_master/images/images/9892881006622/ZS5014-dataSheetMain.pdf)

- [74LS14 Hex Schmitt Trigger IC](https://www.jaycar.co.nz/74ls14-hex-schmitt-trigger-ic/p/ZS5014)

###  RS422 / RS485 Shield for Arduino UNO 

https://store-usa.arduino.cc/collections/home-industrial-automation/products/rs422-rs485-shield-for-arduino-uno
