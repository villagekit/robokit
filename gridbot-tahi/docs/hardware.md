# Grid Bot Tahi hardware

## Limit Switches

opto = 24v -> 3.3v opto

- mcu D2 : opto O1
- mcu D4 : opto O2
- mcu 3.3v : opto VCC
- mcu GND : opto GND

- opto 1+ -> length axis min limit switch C (Common) / 1
- opto 1- -> 24V GND
- 24V -> length axis min limit switch NC (Normally Connected) / 3

- opto 2+ -> length axis max limit switch C (Common) / 1
- opto 2- -> 24V GND
- 24V -> length axis max limit switch NC (Normally Connected) / 3

```
graph LR
    subgraph Optocoupler
        direction LR

        subgraph Input Side
            OptoInput1+[1+]
            OptoInput1-[1-]

            OptoInput2+[2+]
            OptoInput2-[2-]

            OptoInput3+[3+]
            OptoInput3-[3-]

            OptoInput4+[4+]
            OptoInput4-[4-]

            OptoVCC[VCC]
            OptoGND[GND]

            OptoInput
        end

        OptoInput[Input side] --> OptoOutput[Output side]

        subgraph Output Side
            OptoOutput1[O1]
            OptoOutput2[O2]
            OptoOutput3[O3]
            OptoOutput4[O4]

            OptoOutput
        end
    end

    subgraph Length Axis Min Limit Switch
        LengthMinC[Common]
        LengthMinNC[NC]
    end

    subgraph Length Axis Max Limit Switch
        LengthMaxC[Common]
        LengthMaxNC[NC]
    end

    subgraph Width Axis Min Limit Switch
        WidthMinC[Common]
        WidthMinNC[NC]
    end

    subgraph Width Axis Max Limit Switch
        WidthMaxC[Common]
        WidthMaxNC[NC]
    end

    subgraph Microcontroller
        McuD2[D2]
        McuD4[D4]
        McuD5[D5?]
        McuD6[D6?]
    end
     
    24VCC[24V VCC] --> LengthMinC
    24VCC[24V VCC] --> LengthMaxC
    24VCC[24V VCC] --> WidthMinC
    24VCC[24V VCC] --> WidthMaxC

    LengthMinNC --> OptoInput1+
    LengthMaxNC --> OptoInput2+
    WidthMinNC --> OptoInput3+
    WidthMaxNC --> OptoInput4+

    24GND[24V Ground] --> OptoInput1-
    24GND[24V Ground] --> OptoInput2-
    24GND[24V Ground] --> OptoInput3-
    24GND[24V Ground] --> OptoInput4-

    3.3VCC[3.3V VCC] --> OptoVCC
    3.3GND[3.3V GND] --> OptoGND

    OptoOutput1 --> McuD2
    OptoOutput2 --> McuD4
    OptoOutput3 --> McuD5
    OptoOutput4 --> McuD6
```

## Stepper Driver: 
