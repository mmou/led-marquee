#!/bin/bash
sudo ./demo -t 10 -D 1 runtext16.ppm --led-no-hardware-pulse --led-gpio-mapping=adafruit-hat --led-rows=16 --led-cols=32 --led-chain=2 --led-multiplexing=3 --led-row-addr-type=2 --led-brightness=50
