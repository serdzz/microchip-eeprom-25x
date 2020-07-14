# Microchip 25x embedded-hal SPI driver crate

![](https://img.shields.io/crates/v/microchip-eeprom-25x.svg)
![](https://docs.rs/microchip-eeprom-25x/badge.svg)

Rust HAL implementation (using SPI drivers) for Microchip's 25 series EEPROM chips.

Supported models:

| Model     | Datasheet                                                                                                       | Density (bits) | Page size (bytes) |
|-----------|-----------------------------------------------------------------------------------------------------------------|----------------|-------------------|
| 25LC080C  | [Link](http://ww1.microchip.com/downloads/en/DeviceDoc/25LC080C-080D-160C-160D-320A-640A-128-256-20002131D.pdf) | 8k             | 16                |
| 25LC080D  | [Link](http://ww1.microchip.com/downloads/en/DeviceDoc/25LC080C-080D-160C-160D-320A-640A-128-256-20002131D.pdf) | 8k             | 32                |
| 25LC160C  | [Link](http://ww1.microchip.com/downloads/en/DeviceDoc/25LC080C-080D-160C-160D-320A-640A-128-256-20002131D.pdf) | 16k            | 16                |
| 25LC160D  | [Link](http://ww1.microchip.com/downloads/en/DeviceDoc/25LC080C-080D-160C-160D-320A-640A-128-256-20002131D.pdf) | 16k            | 32                |
| 25LC320A  | [Link](http://ww1.microchip.com/downloads/en/DeviceDoc/25LC080C-080D-160C-160D-320A-640A-128-256-20002131D.pdf) | 32k            | 32                |
| 25LC640A  | [Link](http://ww1.microchip.com/downloads/en/DeviceDoc/25LC080C-080D-160C-160D-320A-640A-128-256-20002131D.pdf) | 64k            | 32                |
| 25LC128   | [Link](http://ww1.microchip.com/downloads/en/DeviceDoc/25LC080C-080D-160C-160D-320A-640A-128-256-20002131D.pdf) | 128k           | 64                |
| 25LC256   | [Link](http://ww1.microchip.com/downloads/en/DeviceDoc/25LC080C-080D-160C-160D-320A-640A-128-256-20002131D.pdf) | 256k           | 64                |
| 25LC512   | [Link](http://ww1.microchip.com/downloads/en/DeviceDoc/22065C.pdf)                                              | 512k           | 128               |
| 25LC1024  | [Link](http://ww1.microchip.com/downloads/en/DeviceDoc/22064D.pdf)                                              | 1024k          | 256               |

## Usage

Include [library](https://crates.io/crates/microchip-eeprom-25x) as a dependency in your Cargo.toml

```
[dependencies.microchip-eeprom-25x]
version = "<version>"
```

Use embedded-hal implementation to get SPI and a GPIO OutputPin for the hold line, write protect line and chip select.


