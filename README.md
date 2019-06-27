# hub75 marquee

## System description





## How to build on linux for raspberry pi
1. Install the cross-compiler:
```
apt install gcc-arm-linux-gnueabihf
```

1. Install the target:
```
rustup target add armv7-unknown-linux-gnueabihf
``` 

1. Add a `./cargo/config` file with the following contents:
```
[target.armv7-unknown-linux-gnueabihf]
linker = "arm-linux-gnueabihf-gcc"
```

1. Build for rpi:
```
cargo build --target armv7-unknown-linux-gnueabihf
```

1. The binary can be found at
   `target/armv7-unknown-linux-gnueabihf/debug/marquee` 

## Commands that work
```
sudo ./demo -t 10 -D 1 runtext16.ppm --led-no-hardware-pulse
--led-gpio-mapping=adafruit-hat --led-rows=16 --led-cols=32 --led-cha
in=2 --led-multiplexing=3 --led-row-addr-type=2 --led-brightness=50
```
