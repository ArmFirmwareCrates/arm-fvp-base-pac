// SPDX-FileCopyrightText: Copyright 2023-2024 Arm Limited and/or its affiliates <open-source-office@arm.com>
// SPDX-License-Identifier: MIT OR Apache-2.0

//! # Peripheral Access Crate fro Arm Fixed Virtual Platform
//!
//! The crate provides access to the peripherals of [Arm Fixed Virtual Platform](https://developer.arm.com/Tools%20and%20Software/Fixed%20Virtual%20Platforms).

#![no_std]

use core::{
    fmt::{self, Debug, Formatter},
    marker::PhantomData,
};

use arm_gic::GICDRegisters;
use arm_pl011_uart::PL011Registers;
use arm_sp805::SP805Registers;
use spin::mutex::Mutex;

static PERIPHERALS_TAKEN: Mutex<bool> = Mutex::new(false);

/// The physical instance of some device's MMIO space.
pub struct PhysicalInstance<T> {
    pa: usize,
    _phantom: PhantomData<T>,
}

impl<T> Debug for PhysicalInstance<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("PhysicalInstance")
            .field("pa", &self.pa)
            .field("size", &size_of::<T>())
            .finish()
    }
}

impl<T> PhysicalInstance<T> {
    /// # Safety
    ///
    /// This must refer to the physical address of a real set of device registers of type `T`, and
    /// there must only ever be a single `PhysicalInstance` for those device registers.
    pub unsafe fn new(pa: usize) -> Self {
        Self {
            pa,
            _phantom: PhantomData,
        }
    }

    /// Returns the physical base address of the device's registers.
    pub fn pa(&self) -> usize {
        self.pa
    }
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
            gicd: PhysicalInstance::new(0x2f00_0000),
        }
    }
}
