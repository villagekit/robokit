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

- Pin 1: D0 / PG_9
- Pin 2: D1 / PG_14
- Pin 3: D2 / PF_15
- Pin 4: D4 / PF_14

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
        McuPin0[PIN_0]
        McuPin1[PIN_1]
        McuPin2[PIN_2]
        McuPin3[PIN_3]
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

    OptoOutput1 --> McuPin0
    OptoOutput2 --> McuPin1
    OptoOutput3 --> McuPin2
    OptoOutput4 --> McuPin3
```

## Stepper Driver

Length Axis:

- PIN_0: PG_1
- PIN_1: PF_9

Width Axis

- PIN_0: PF_7
- PIN_0: PF_8

```
graph LR
    subgraph Microcontroller [MCU]
        MCU_GND[GND]
        MCU_PIN_0[PIN_0]
        MCU_PIN_1[PIN_1]
    end

    subgraph Stepper Motor Driver
        DRIVER_PUL+[PUL+]
        DRIVER_PUL-[PUL-]
        DRIVER_DIR+[DIR+]
        DRIVER_DIR-[DIR-]
        DRIVER_ENBL-[ENBL-]
        DRIVER_ENBL+[ENBL-]
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

    MCU_PIN_0--> DRIVER_PUL+[PUL+]
    MCU_PIN_1 --> DRIVER_DIR+[DIR+]
    MCU_GND --> DRIVER_PUL-[PUL-]
    MCU_GND --> DRIVER_DIR-[DIR-]
    MCU_GND --> DRIVER_ENBL-[ENBL-]

    48V_VCC[48V VCC] --> DRIVER_DC+[DC+]
    48V_GND[48V GND] --> DRIVER_DC-[DC-]

    DRIVER_A+[A+] --- MOTOR_RED[red]
    DRIVER_A-[A-] --- MOTOR_GREEN[green]
    DRIVER_B+[B+] --- MOTOR_YELLOW[yellow]
    DRIVER_B-[B-] --- MOTOR_BLUE[blue]
```
