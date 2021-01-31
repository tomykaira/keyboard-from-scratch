# My keyboard firmware

This is USB HID keyboard firmware implementation for FLAT6 (my personal keyboard).

This work refers [KOBA789's work](https://github.com/KOBA789/keyboard-from-scratch), but most of the firmware implementation is my own. Do not send issues and requests to the original repo.

- Current implementation is not compat with dapboot. Do not use DFU for bluepill (STM32F103). Use ST-LinkV2 instead. It will be ok if DFU is implemented by the hardware, e.g. STM32F042.
