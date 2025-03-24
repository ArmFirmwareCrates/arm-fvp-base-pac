// SPDX-FileCopyrightText: Copyright 2023-2025 Arm Limited and/or its affiliates <open-source-office@arm.com>
// SPDX-License-Identifier: MIT OR Apache-2.0

//! # Peripheral Access Crate fro Arm Fixed Virtual Platform
//!
//! The crate provides access to the peripherals of [Arm Fixed Virtual Platform](https://developer.arm.com/Tools%20and%20Software/Fixed%20Virtual%20Platforms).

#![no_std]

// Re-export peripheral drivers and common safe-mmio types
pub use arm_gic;
pub use arm_pl011_uart;
pub use arm_sp805;
pub use safe_mmio::{PhysicalInstance, UniqueMmioPointer};
pub mod power_controller;

use arm_gic::GICDRegisters;
use arm_pl011_uart::PL011Registers;
use arm_sp805::SP805Registers;
use core::{fmt::Debug, ops::RangeInclusive};
use power_controller::FvpPowerControllerRegisters;
use spin::mutex::Mutex;

static PERIPHERALS_TAKEN: Mutex<bool> = Mutex::new(false);

/// Memory map based on 'Table 3-3 Base Platform memory map'.
pub struct MemoryMap;

impl MemoryMap {
    pub const TRUSTED_BOOT_ROM: RangeInclusive<usize> = 0x00_0000_0000..=0x00_03FF_FFFF;
    pub const TRUSTED_SRAM: RangeInclusive<usize> = 0x00_0400_0000..=0x00_0403_FFFF;
    pub const TRUSTED_DRAM: RangeInclusive<usize> = 0x00_0600_0000..=0x00_07FF_FFFF;
    pub const NOR_FLASH0: RangeInclusive<usize> = 0x00_0800_0000..=0x00_0BFF_FFFF;
    pub const NOR_FLASH1: RangeInclusive<usize> = 0x00_0C00_0000..=0x00_0FFF_FFFF;
    pub const PSRAM: RangeInclusive<usize> = 0x00_1400_0000..=0x00_17FF_FFFF;
    pub const VRAM: RangeInclusive<usize> = 0x00_1800_0000..=0x00_19FF_FFFF;
    pub const NON_TRUSTED_ROM: RangeInclusive<usize> = 0x00_1F00_0000..=0x00_1F00_0FFF;
    pub const NON_TRUSTED_SRAM: RangeInclusive<usize> = 0x00_2E00_0000..=0x00_2E00_FFFF;
    pub const DRAM0: RangeInclusive<usize> = 0x00_8000_0000..=0x00_FFFF_FFFF;
    pub const DRAM1: RangeInclusive<usize> = 0x08_8000_0000..=0x0F_FFFF_FFFF;
    pub const DRAM2: RangeInclusive<usize> = 0x88_0000_0000..=0xFF_FFFF_FFFF;
}

/// FVP peripherals
#[derive(Debug)]
pub struct Peripherals {
    pub uart0: PhysicalInstance<PL011Registers>,
    pub uart1: PhysicalInstance<PL011Registers>,
    pub uart2: PhysicalInstance<PL011Registers>,
    pub uart3: PhysicalInstance<PL011Registers>,
    pub watchdog: PhysicalInstance<SP805Registers>,
    pub gicd: PhysicalInstance<GICDRegisters>,
    pub power_controller: PhysicalInstance<FvpPowerControllerRegisters>,
}

impl Peripherals {
    /// Take the peripherals once
    pub fn take() -> Option<Self> {
        if !*PERIPHERALS_TAKEN.lock() {
            // SAFETY: PERIPHERALS_TAKEN ensures that this is only called once.
            Some(unsafe { Self::steal() })
        } else {
            None
        }
    }

    /// Unsafe version of take()
    ///
    /// # Safety
    ///
    /// The caller must ensure that each peripheral is only used once.
    pub unsafe fn steal() -> Self {
        *PERIPHERALS_TAKEN.lock() = true;

        Peripherals {
            uart0: PhysicalInstance::new(0x1c09_0000),
            uart1: PhysicalInstance::new(0x1c0a_0000),
            uart2: PhysicalInstance::new(0x1c0b_0000),
            uart3: PhysicalInstance::new(0x1c0c_0000),
            watchdog: PhysicalInstance::new(0x1c0f_0000),
            power_controller: PhysicalInstance::new(0x1c10_0000),
            gicd: PhysicalInstance::new(0x2f00_0000),
        }
    }
}
