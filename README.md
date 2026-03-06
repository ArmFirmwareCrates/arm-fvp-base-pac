# Arm Base Fixed Virtual Platform Peripheral Access Crate

This crate provides peripheral access for the [Arm Fixed Virtual Platform](https://developer.arm.com/Tools%20and%20Software/Fixed%20Virtual%20Platforms),
**specifically for the Base platform FVPs**.

The implementation is based on [Fast Models Fixed Virtual Platforms Reference Guide Revision: 11.28](https://developer.arm.com/documentation/100966/1128)
and [Fast Models Reference Guide Revision: 11.28](https://developer.arm.com/documentation/100964/1128/).

## Implemented features

* Memory map description.
* `Peripherals` structure for obtaining individual peripherals while maintaining ownership.
* FVP power controller driver
* FVP system peripheral driver
* Re-exporting `arm-cci`, `arm-generic-timer`, `arm-gic`, `arm-pl011-uart`, `arm-sp805` and
  `arm-tzc` drivers and common `safe-mmio` types. This enables projects to use these peripherals
  without adding these crates as explicit dependencies to the project. This also prevents having
  driver version conflicts in the project.

## Feature flags

* `base-revc`: Enables the Base Platform RevC-only MMIO regions and interrupt assignments.
* `base-r`: Selects a BaseR Platform compatible memory map, by swapping the first two 2 GiB blocks of address space.
* `el1`: Enables system register based GIC and Timers features which relies on EL1 system registers.
* `el2`: Enables system register based GIC and Timers features which relies on EL2 system registers.
* `el3`: Enables system register based GIC and Timers features which relies on EL3 system registers.
* `fakes`: Accesses fake system registers rather than the real ones in GIC and Generic Timer drivers,
  for running tests on the host.

## Limitations

* Not all peripherals are handled yet.

## License

The project is MIT and Apache-2.0 dual licensed, see `LICENSE-APACHE` and `LICENSE-MIT`.

## Maintainers

arm-fvp-base-pac is a trustedfirmware.org maintained project. All contributions are ultimately merged by
the maintainers listed below.

* Bálint Dobszay <balint.dobszay@arm.com>
  [balint-dobszay-arm](https://github.com/balint-dobszay-arm)
* Imre Kis <imre.kis@arm.com>
  [imre-kis-arm](https://github.com/imre-kis-arm)
* Sandrine Afsa <sandrine.afsa@arm.com>
  [sandrine-bailleux-arm](https://github.com/sandrine-bailleux-arm)

## Contributing

Please follow the directions of the [Trusted Firmware Processes](https://trusted-firmware-docs.readthedocs.io/en/latest/generic_processes/index.html).

Contributions are handled through [review.trustedfirmware.org](https://review.trustedfirmware.org/q/project:arm-firmware-crates/arm-fvp-base-pac).

## Arm trademark notice

Arm is a registered trademark of Arm Limited (or its subsidiaries or affiliates).

This project uses some of the Arm product, service or technology trademarks, as listed in the
[Trademark List][1], in accordance with the [Arm Trademark Use Guidelines][2].

Subsequent uses of these trademarks throughout this repository do not need to be prefixed with the
Arm word trademark.

[1]: https://www.arm.com/company/policies/trademarks/arm-trademark-list
[2]: https://www.arm.com/company/policies/trademarks/guidelines-trademarks

--------------

*Copyright The arm-fvp-base-pac Contributors.*

*Arm is a registered trademark of Arm Limited (or its subsidiaries or affiliates).*
