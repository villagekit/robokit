# gridbot-software

## micro-controller

ST Nucleo F767ZI

- [platform.io page](https://docs.platformio.org/en/latest/boards/ststm32/nucleo_f767zi.html)
- [datasheet](http://www.farnell.com/datasheets/2200746.pdf)
- [user guide](https://www.st.com/content/ccc/resource/technical/document/user_manual/group0/26/49/90/2e/33/0d/4a/da/DM00244518/files/DM00244518.pdf/jcr:content/translations/en.DM00244518.pdf)

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