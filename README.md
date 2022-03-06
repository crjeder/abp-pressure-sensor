# hx711_spi

![License](https://img.shields.io/crates/l/hx711_spi?style=plastic)

This is a platform agnostic driver to interface with Honeywells APB line of pressure sensors (https://sps.honeywell.com/gb/en/products/advanced-sensing-technologies/healthcare-sensing/board-mount-pressure-sensors/basic-abp-series)
This `[no_std]` driver is built using [`embedded-hal`][2] traits.

## Usage
It is recommended to always use [cargo-crev](https://github.com/crev-dev/cargo-crev)
to verify the trustworthiness of each of your dependencies, including this one.

Use an embedded-hal implementation to get I2C.


## Example

### Wiring

### Code

## What works
(tested on Raspberry Pi)

  -

## TODO

  - [ ] Test on more platforms
  -


## Feedback
All kind of feedback is welcome. If you have questions or problems, please post them on the issue tracker
This is literally the first code I ever wrote in rust. I am still learning. So please be patient, it might take me some time to fix a bug. I may have to break my knowledge sound-barrier.
If you have tested on another platform I'd like to hear about that, too!

# References

  - [datasheet][1]

[1]: https://prod-edam.honeywell.com/content/dam/honeywell-edam/sps/siot/en-gb/products/sensors/pressure-sensors/board-mount-pressure-sensors/basic-abp-series/documents/sps-siot-basic-board-mount-pressure-abp-series-datasheet-32305128-ciid-155789.pdf

  - [embedded-hal][2]

[2]: https://github.com/rust-embedded/embedded-hal

## License

Licensed under either of

  - Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
  - MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
