# Grid Bot Tahi hardware

## Microcontroller

- micro-controller: [Nucleo-144 STM32-F767ZI](https://nz.element14.com/stmicroelectronics/nucleo-f767zi/dev-board-nucleo-32-mcu/dp/2546569)
  - pins: https://os.mbed.com/platforms/ST-Nucleo-F767ZI/
  - chip: https://www.st.com/resource/en/datasheet/stm32f767zi.pdf

Notes:

- some pins overlap with Cargo Flash / USB thing.
  - which pins are those?
    - D3? ( PE-13)
      - but why?

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
        McuD2[D0]
        McuD4[D1]
        McuD5[D2]
        McuD6[D3]
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

## Stepper Driver

Length Axis

```
graph LR
    subgraph Microcontroller [MCU]
        MCU_GND[GND]
        MCU_D0[D0]
        MCU_D1[D1]
    end

    subgraph Stepper Motor Driver
        DRIVER_PUL+[PUL+]
        DRIVER_PUL-[PUL-]
        DRIVER_DIR+[DIR+]
        DRIVER_DIR-[DIR-]
        DRIVER_ENBL-[ENBL-]
        DRIVER_DC+[DC+]
        DRIVER_DC-[DC-]
        DRIVER_A+[A+]
        DRIVER_A-[A-]
        DRIVER_B+[B+]
        DRIVER_B-[B-]
    end

    subgraph Stepper Motor
        MOTOR_RED[red]
        MOTOR_GREEN[green]
        MOTOR_YELLOW[yellow]
        MOTOR_BLUE[blue]
    end

    MCU_D0--> DRIVER_PUL+[PUL+]
    MCU_GND --> DRIVER_PUL-[PUL-]
    MCU_D1 --> DRIVER_DIR+[DIR+]
    MCU_GND --> DRIVER_DIR-[DIR-]
    MCU_GND --> DRIVER_ENBL-[ENBL-]

    48V_POS[48V] --> DRIVER_DC+[DC+]
    48V_NEG[48V GND] --> DRIVER_DC-[DC-]

    DRIVER_A+[A+] --- MOTOR_RED[red]
    DRIVER_A-[A-] --- MOTOR_GREEN[green]
    DRIVER_B+[B+] --- MOTOR_YELLOW[yellow]
    DRIVER_B-[B-] --- MOTOR_BLUE[blue]
```

Width Axis

```
graph LR
    subgraph Microcontroller [MCU]
        MCU_GND[GND]
        MCU_D0[D0]
        MCU_D1[D1]
    end

    subgraph Stepper Motor Driver
        DRIVER_PUL+[PUL+]
        DRIVER_PUL-[PUL-]
        DRIVER_DIR+[DIR+]
        DRIVER_DIR-[DIR-]
        DRIVER_ENBL-[ENBL-]
        DRIVER_DC+[DC+]
        DRIVER_DC-[DC-]
        DRIVER_A+[A+]
        DRIVER_A-[A-]
        DRIVER_B+[B+]
        DRIVER_B-[B-]
    end

    subgraph Stepper Motor
        MOTOR_RED[red]
        MOTOR_GREEN[green]
        MOTOR_YELLOW[yellow]
        MOTOR_BLUE[blue]
    end

    MCU_D0--> DRIVER_PUL+[PUL+]
    MCU_GND --> DRIVER_PUL-[PUL-]
    MCU_D1 --> DRIVER_DIR+[DIR+]
    MCU_GND --> DRIVER_DIR-[DIR-]
    MCU_GND --> DRIVER_ENBL-[ENBL-]

    48V_POS[48V] --> DRIVER_DC+[DC+]
    48V_NEG[48V GND] --> DRIVER_DC-[DC-]

    DRIVER_A+[A+] --- MOTOR_RED[red]
    DRIVER_A-[A-] --- MOTOR_GREEN[green]
    DRIVER_B+[B+] --- MOTOR_YELLOW[yellow]
    DRIVER_B-[B-] --- MOTOR_BLUE[blue]
```
