# Hardware Notes

## Concepts

- [Electrical ("Galvanic") Isolation](https://en.wikipedia.org/wiki/Galvanic_isolation)
- Pull-up resistor (10k Ω)
- Capacitor as low-pass filter (100nF)
- Schmitt Trigger
- Inline resistor between sensor and mcu (120? Ω)

## Components

### Opto-coupler

https://www.aliexpress.com/item/32651786932.html

```
graph LR
    subgraph Optocoupler
        direction LR

        subgraph Input Stage
            Input+ --> R1[Resistor #1] --> OI
            Input+ --> R2[Resistor #2] --> D1[Input Status LED]
            Input- --> D1
            Input- --> OI
        end

        OI[Optocoupler - LED side] --> OO[Optocoupler - Phototransistor side]

        subgraph Output Stage
            OO --> OVCC[Output VCC]
            OO --> Output
            Output --> D2[Output Status LED] --> R3[Resistor #3] --> OGND[Output Ground]
            Output --> R4[Resistor #4] --> OGND
        end
    end
```

Where input is 5V and output is 24V:

- R1 = 270 Ω
- R2 = 270 Ω
- R3 = 2.2K Ω
- R4 = 10K Ω

Where input is 3.3V and output is 24V:

- R1 = 330 Ω
- R2 = 330 Ω
- R3 = 3.9K Ω
- R4 = 100K Ω

Where input is 24V and output is 3.3V:

- R1 = 2.2K Ω
- R2 = 3.9K Ω
- R3 = 330 Ω
- R4 = 10K Ω

### Limit Switch

With opto-coupler above:

```
graph LR
    subgraph Optocoupler
        direction LR

        subgraph Input Stage
            Input+
            Input-

            OI
        end

        OI[Optocoupler - LED side] --> OO[Optocoupler - Phototransistor side]

        subgraph Output Stage
            Output
            OO
        end
    end

    subgraph Limit Switch
        LSC[Limit Switch Common]
        LSNC[Limit Switch NC]
    end

    subgraph Microcontroller
        MInput[GPIO Input]
    end
     
    IVCC[Input VCC] --> LSC
    LSNC --> Input+
    IGND[Input Ground] --> Input-
    Output --> MInput
```

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
