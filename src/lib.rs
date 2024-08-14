// SPDX-FileCopyrightText: Copyright 2023-2024 Arm Limited and/or its affiliates <open-source-office@arm.com>
// SPDX-License-Identifier: MIT OR Apache-2.0

//! # Peripheral Access Crate fro Arm Fixed Virtual Platform
//!
//! The crate provides access to the peripherals of [Arm Fixed Virtual Platform](https://developer.arm.com/Tools%20and%20Software/Fixed%20Virtual%20Platforms).

#![no_std]

use core::{marker::PhantomData, ops::Deref};

use spin::mutex::Mutex;

use arm_gic::GICDRegisters;
use arm_pl011::PL011Registers;
use arm_sp805::SP805Registers;

/// UART0 - PL011
pub struct UART0 {
    _marker: PhantomData<*const ()>,
}

impl UART0 {
    pub const PTR: *const PL011Registers = 0x1c09_0000 as *const _;
}

impl Deref for UART0 {
    type Target = PL011Registers;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::PTR }
    }
}

unsafe impl Send for UART0 {}

/// UART1 - PL011
pub struct UART1 {
    _marker: PhantomData<*const ()>,
}

impl UART1 {
    pub const PTR: *const PL011Registers = 0x1c0a_0000 as *const _;
}

impl Deref for UART1 {
    type Target = PL011Registers;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::PTR }
    }
}

unsafe impl Send for UART1 {}

/// UART2 - PL011
pub struct UART2 {
    _marker: PhantomData<*const ()>,
}

impl UART2 {
    pub const PTR: *const PL011Registers = 0x1c0b_0000 as *const _;
}

impl Deref for UART2 {
    type Target = PL011Registers;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::PTR }
    }
}

unsafe impl Send for UART2 {}

/// UART3 - PL011
pub struct UART3 {
    _marker: PhantomData<*const ()>,
}

impl UART3 {
    pub const PTR: *const PL011Registers = 0x1c0c_0000 as *const _;
}

impl Deref for UART3 {
    type Target = PL011Registers;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::PTR }
    }
}

unsafe impl Send for UART3 {}

/// Watchdog - SP805
#[allow(clippy::upper_case_acronyms)]
pub struct WATCHDOG {
    _marker: PhantomData<*const ()>,
}

impl WATCHDOG {
    pub const PTR: *const SP805Registers = 0x1c0f_0000 as *const _;
}

impl Deref for WATCHDOG {
    type Target = SP805Registers;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::PTR }
    }
}

unsafe impl Send for WATCHDOG {}

/// GIC Distributor
#[allow(clippy::upper_case_acronyms)]
pub struct GICD {
    _marker: PhantomData<*const ()>,
}

impl GICD {
    pub const PTR: *const GICDRegisters = 0x2f00_0000 as *const _;
}

impl Deref for GICD {
    type Target = GICDRegisters;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::PTR }
    }
}

unsafe impl Send for GICD {}

static PERIPHERALS_TAKEN: Mutex<bool> = Mutex::new(false);

/// FVP peripherals
#[allow(non_snake_case)]
pub struct Peripherals {
    pub UART0: UART0,
    pub UART1: UART1,
    pub UART2: UART2,
    pub UART3: UART3,
    pub WATCHDOG: WATCHDOG,
    pub GICD: GICD,
}

impl Peripherals {
    /// Take the peripherals once
    pub fn take() -> Option<Self> {
        if !*PERIPHERALS_TAKEN.lock() {
            Some(unsafe { Self::steal() })
        } else {
            None
        }
    }

    /// Unsafe version of take()
    ///
    /// # Safety
    /// The caller has to ensure that each peripheral is only used once.
    pub unsafe fn steal() -> Self {
        *PERIPHERALS_TAKEN.lock() = true;

        Peripherals {
            UART0: UART0 {
                _marker: PhantomData,
            },
            UART1: UART1 {
                _marker: PhantomData,
            },
            UART2: UART2 {
                _marker: PhantomData,
            },
            UART3: UART3 {
                _marker: PhantomData,
            },
            WATCHDOG: WATCHDOG {
                _marker: PhantomData,
            },
            GICD: GICD {
                _marker: PhantomData,
            },
        }
    }
}
