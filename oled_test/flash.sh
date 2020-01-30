#!/bin/bash
EXE_PATH=target/thumbv6m-none-eabi/debug
arm-none-eabi-objcopy -O binary $EXE_PATH/oled-test $EXE_PATH/fw.bin
st-flash --reset write $EXE_PATH/fw.bin 0x08000000