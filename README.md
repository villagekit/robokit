# gridbot-software

## micro-controller

ST Nucleo F767ZI

- [platform.io page](https://docs.platformio.org/en/latest/boards/ststm32/nucleo_f767zi.html)
- [board documentation](https://www.st.com/en/evaluation-tools/nucleo-f767zi.html#documentation)
- [chip documentation](https://www.st.com/en/microcontrollers-microprocessors/stm32f767zi.html#documentation)

### dependencies

- Platform.IO
- follow ST-Link [driver instructions](https://docs.platformio.org/en/latest/plus/debug-tools/stlink.html)
- [ST-Link helper](https://github.com/stlink-org/stlink/releases)

## notes

- timer interrupt error handling
  - watchdog timer
    - "if we don't reset the watchdog within X microseconds, something is broken!"
    - https://github.com/stm32duino/Arduino_Core_STM32/tree/master/libraries/IWatchdog
  - set a flag at the end of the timer interrupt
    - if not 
- timer interrupt queue
  - timer interrupts queues events to the main loop, which processes them
  - the less code you share between the isrs and the main loop, the better 
    - sharing mutable state with isrs is a scary thing
  - don't handroll the queue, find a good library that is specific for thread (isr) safe
    - e.g. ringbuffer, (rust: bbqueue), atomic
    - or use a standard queue library
      - or disable the interrupts, read (and copy) the queue, then re-enable the interrupts
- if trying to be very fast, avoid if branches
  - if branches will clear instruction pipeline
- g-code: command queue
  - example:
    - move X motor to 20mm
    - move X motor to 0mm
    - wait 1 second
    - move Y motor to 4mm
  - requirements:
    - when is a command finished?
- fast motors
  - https://stackoverflow.com/questions/62217872/generate-a-fixed-number-of-pulses-on-the-stm32f4-pwm
  - https://www.st.com/en/microcontrollers-microprocessors/stm32f767zi.html#documentation
  - file:///tmp/mozilla_dinosaur0/dm00236305-generalpurpose-timer-cookbook-for-stm32-microcontrollers-stmicroelectronics.pdf
  - https://electronics.stackexchange.com/questions/312926/dynamically-change-pwm-frequency-with-interrupt-with-stm32
  - https://github.com/gin66/FastAccelStepper/blob/master/src/StepperISR_esp32.cpp
  - https://github.com/simplefoc/Arduino-FOC/blob/40336919658f35790c96164dc37a6395fe8b4526/src/drivers/hardware_specific/stm32_mcu.cpp
  - https://github.com/stm32duino/wiki/wiki/HardwareTimer-library
  - https://www.stm32duino.com/viewtopic.php?t=1023
  - https://www.stm32duino.com/viewtopic.php?t=438
- acceleration ramp
  - https://github.com/braun-embedded/ramp-maker/blob/main/src/trapezoidal.rs