// SPDX-FileCopyrightText: Copyright The arm-fvp-base-pac Contributors.
// SPDX-License-Identifier: MIT OR Apache-2.0

#![no_std]
#![doc = include_str!("../README.md")]
#![deny(clippy::undocumented_unsafe_blocks)]

pub mod power_controller;
pub mod system;

// Re-export peripheral drivers and common safe-mmio types
pub use arm_cci;
pub use arm_generic_timer;
pub use arm_gic;
pub use arm_pl011_uart;
pub use arm_sp805;
pub use safe_mmio::{PhysicalInstance, UniqueMmioPointer};

use arm_cci::Cci5x0Registers;
use arm_generic_timer::{CntBase, CntControlBase, CntCtlBase, CntReadBase};
use arm_gic::gicv3::registers::{Gicd, GicrSgi};
use arm_pl011_uart::PL011Registers;
use arm_sp805::SP805Registers;
use core::{fmt::Debug, ops::RangeInclusive};
use power_controller::FvpPowerControllerRegisters;
use spin::mutex::Mutex;
use system::FvpSystemRegisters;

static PERIPHERALS_TAKEN: Mutex<bool> = Mutex::new(false);

/// Memory map based on 'Table 6-5: Base Platform memory map' of 'Fast Models Version 11.28
/// Reference Guide'. The ranges for normal memory regions are public, however the peripheral
/// regions are private and they should be accessed through the `Peripherals` structure.
pub struct MemoryMap;

#[allow(unused)]
impl MemoryMap {
    pub const TRUSTED_BOOT_ROM: RangeInclusive<usize> = 0x00_0000_0000..=0x00_03FF_FFFF;
    pub const TRUSTED_SRAM: RangeInclusive<usize> = 0x00_0400_0000..=0x00_0407_FFFF;
    pub const TRUSTED_DRAM: RangeInclusive<usize> = 0x00_0600_0000..=0x00_07FF_FFFF;
    pub const NOR_FLASH0: RangeInclusive<usize> = 0x00_0800_0000..=0x00_0BFF_FFFF;
    pub const NOR_FLASH1: RangeInclusive<usize> = 0x00_0C00_0000..=0x00_0FFF_FFFF;
    pub const PSRAM: RangeInclusive<usize> = 0x00_1400_0000..=0x00_17FF_FFFF;
    pub const VRAM: RangeInclusive<usize> = 0x00_1800_0000..=0x00_19FF_FFFF;
    const ETHERNET: RangeInclusive<usize> = 0x00_1A00_0000..=0x00_1AFF_FFFF;
    const USB: RangeInclusive<usize> = 0x00_1B00_0000..=0x00_1BFF_FFFF;
    const VE_SYSTEM: RangeInclusive<usize> = 0x00_1C01_0000..=0x00_1C01_FFFF;
    const SYSTEM_CONTROLLER: RangeInclusive<usize> = 0x00_1C02_0000..=0x00_1C02_FFFF;
    const AACI: RangeInclusive<usize> = 0x00_1C04_0000..=0x00_1C04_FFFF;
    const MCI: RangeInclusive<usize> = 0x00_1C05_0000..=0x00_1C05_FFFF;
    const KMI_KEYBOARD: RangeInclusive<usize> = 0x00_1C06_0000..=0x00_1C06_FFFF;
    const KMI_MOUSE: RangeInclusive<usize> = 0x00_1C07_0000..=0x00_1C07_FFFF;
    const UART0: RangeInclusive<usize> = 0x00_1C09_0000..=0x00_1C09_FFFF;
    const UART1: RangeInclusive<usize> = 0x00_1C0A_0000..=0x00_1C0A_FFFF;
    const UART2: RangeInclusive<usize> = 0x00_1C0B_0000..=0x00_1C0B_FFFF;
    const UART3: RangeInclusive<usize> = 0x00_1C0C_0000..=0x00_1C0C_FFFF;
    const VFS2: RangeInclusive<usize> = 0x00_1C0D_0000..=0x00_1C0D_FFFF;
    const WATCHDOG: RangeInclusive<usize> = 0x00_1C0F_0000..=0x00_1C0F_FFFF;
    const POWER_CONTROLLER: RangeInclusive<usize> = 0x00_1C10_0000..=0x00_1C10_FFFF;
    const DUAL_TIMER0: RangeInclusive<usize> = 0x00_1C11_0000..=0x00_1C11_FFFF;
    const DUAL_TIEMR1: RangeInclusive<usize> = 0x00_1C12_0000..=0x00_1C12_FFFF;
    const VIRTIO_BLOCK_DEVICE: RangeInclusive<usize> = 0x00_1C13_0000..=0x00_1C13_FFFF;
    const VIRTIO_PLAN9_DEVICE: RangeInclusive<usize> = 0x00_1C14_0000..=0x00_1C14_FFFF;
    const VIRTIO_NET_DEVICE: RangeInclusive<usize> = 0x00_1C15_0000..=0x00_1C15_FFFF;
    const RTC: RangeInclusive<usize> = 0x00_1C17_0000..=0x00_1C17_FFFF;
    const CF_CARD: RangeInclusive<usize> = 0x00_1C1A_0000..=0x00_1C1A_FFFF;
    const CLCD_CONTROLLER: RangeInclusive<usize> = 0x00_1C1F_0000..=0x00_1C1F_FFFF;
    const VIRTIO_RNG: RangeInclusive<usize> = 0x00_1C20_0000..=0x00_1C20_FFFF;
    const LS64_TESTING_FIFO: RangeInclusive<usize> = 0x00_1D00_0000..=0x00_1D00_FFFF;
    const UTILITY_BUS: RangeInclusive<usize> = 0x00_1E00_0000..=0x00_1EFF_FFFF;
    pub const NON_TRUSTED_ROM: RangeInclusive<usize> = 0x00_1F00_0000..=0x00_1F00_0FFF;
    const CORESIGHT: RangeInclusive<usize> = 0x00_2000_0000..=0x00_27FF_FFFF;
    const CCI_550: RangeInclusive<usize> = 0x00_2A00_0000..=0x00_2A09_FFFF;
    const REFCLK_CNTCONTROL: RangeInclusive<usize> = 0x00_2A43_0000..=0x00_2A43_FFFF;
    const EL2_WATCHDOG_CONTROL: RangeInclusive<usize> = 0x00_2A44_0000..=0x00_2A44_FFFF;
    const EL2_WATCHDOG_REFRESH: RangeInclusive<usize> = 0x00_2A45_0000..=0x00_2A45_FFFF;
    const TRUSTED_WATCHDOG: RangeInclusive<usize> = 0x00_2A49_0000..=0x00_2A49_FFFF;
    const TRUSTZONE_CONTROLLER: RangeInclusive<usize> = 0x00_2A4A_0000..=0x00_2A4A_FFFF;
    const REFCLK_CNTREAD: RangeInclusive<usize> = 0x00_2A80_0000..=0x00_2A80_FFFF;
    const AP_REFCLK_CNTCTL: RangeInclusive<usize> = 0x00_2A81_0000..=0x00_2A81_FFFF;
    const AP_REFCLK_CNTBASE0: RangeInclusive<usize> = 0x00_2A82_0000..=0x00_2A82_FFFF;
    const AP_REFCLK_CNTBASE1: RangeInclusive<usize> = 0x00_2A83_0000..=0x00_2A83_FFFF;
    const DMC_400_CFG: RangeInclusive<usize> = 0x00_2B0A_0000..=0x00_2B0A_FFFF;
    const SMMUV3_AEM: RangeInclusive<usize> = 0x00_2B40_0000..=0x00_2B4F_FFFF;
    const DMA330X4: RangeInclusive<usize> = 0x00_2B50_0000..=0x00_2B5F_FFFF;
    const GICC: RangeInclusive<usize> = 0x00_2C00_0000..=0x00_2C00_1FFF;
    const GICH: RangeInclusive<usize> = 0x00_2C01_0000..=0x00_2C01_0FFF;
    const GICV: RangeInclusive<usize> = 0x00_2C02_F000..=0x00_2C03_0FFF;
    const CCI_400: RangeInclusive<usize> = 0x00_2C09_0000..=0x00_2C09_FFFF;
    const MALI_G76: RangeInclusive<usize> = 0x00_2D00_0000..=0x00_2DFF_0000;
    pub const NON_TRUSTED_SRAM: RangeInclusive<usize> = 0x00_2E00_0000..=0x00_2E00_FFFF;
    const GICD: RangeInclusive<usize> = 0x00_2F00_0000..=0x00_2F00_FFFF;
    const GITS: RangeInclusive<usize> = 0x00_2F02_0000..=0x00_2F03_FFFF;
    const GICR: RangeInclusive<usize> = 0x00_2F10_0000..=0x00_2F1F_FFFF;
    const PCIE_CONFIG_REGION: RangeInclusive<usize> = 0x00_4000_0000..=0x00_4FFF_FFFF;
    const PCIE_MEMORY_REGION1: RangeInclusive<usize> = 0x00_5000_0000..=0x00_5FFF_FFFF;
    const TRUSTED_RNG: RangeInclusive<usize> = 0x00_7FE6_0000..=0x00_7FE6_0FFF;
    const TRUSTED_NV_COUNTERS: RangeInclusive<usize> = 0x00_7FE7_0000..=0x00_7FE7_0FFF;
    const TRUSTED_ROOT_KEY_STORAGE: RangeInclusive<usize> = 0x00_7FE8_0000..=0x00_7FE8_0FFF;
    const DDR3_PHY: RangeInclusive<usize> = 0x00_7FEF_0000..=0x00_7FEF_FFFF;
    const HDLCD_CONTROLLER: RangeInclusive<usize> = 0x00_7FF6_0000..=0x00_7FF6_FFFF;
    pub const DRAM0: RangeInclusive<usize> = 0x00_8000_0000..=0x00_FFFF_FFFF;
    pub const DRAM1: RangeInclusive<usize> = 0x08_8000_0000..=0x0F_FFFF_FFFF;
    const PCIE_MEMORY_REGION2: RangeInclusive<usize> = 0x40_0000_0000..=0x7F_FFFF_FFFF;
    pub const DRAM2: RangeInclusive<usize> = 0x88_0000_0000..=0xFF_FFFF_FFFF;
    pub const DRAM3: RangeInclusive<usize> = 0x00_0880_0000_0000..=0x00_0FFF_FFFF_FFFF;
    pub const DRAM4: RangeInclusive<usize> = 0x08_8000_0000_0000..=0x0F_FFFF_FFFF_FFFF;
    pub const DRAM5: RangeInclusive<usize> = 0x88_0000_0000_0000..=0x8F_FFFF_FFFF_FFFF;
}

/// FVP peripherals
#[derive(Debug)]
pub struct Peripherals {
    pub system: PhysicalInstance<FvpSystemRegisters>,
    pub uart0: PhysicalInstance<PL011Registers>,
    pub uart1: PhysicalInstance<PL011Registers>,
    pub uart2: PhysicalInstance<PL011Registers>,
    pub uart3: PhysicalInstance<PL011Registers>,
    pub watchdog: PhysicalInstance<SP805Registers>,
    pub power_controller: PhysicalInstance<FvpPowerControllerRegisters>,
    /// CCI-550 is only available on the FVP Base RevC platform when the cluster count is 1 or 2.
    pub cci_550: PhysicalInstance<Cci5x0Registers>,
    pub refclk_cntcontrol: PhysicalInstance<CntControlBase>,
    pub trusted_watchdog: PhysicalInstance<SP805Registers>,
    pub refclk_cntread: PhysicalInstance<CntReadBase>,
    pub ap_refclk_cntctl: PhysicalInstance<CntCtlBase>,
    pub ap_refclk_cntbase0: PhysicalInstance<CntBase>,
    pub ap_refclk_cntbase1: PhysicalInstance<CntBase>,
    pub gicd: PhysicalInstance<Gicd>,
    pub gicr: PhysicalInstance<GicrSgi>,
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
            system: PhysicalInstance::new(*MemoryMap::VE_SYSTEM.start()),
            uart0: PhysicalInstance::new(*MemoryMap::UART0.start()),
            uart1: PhysicalInstance::new(*MemoryMap::UART1.start()),
            uart2: PhysicalInstance::new(*MemoryMap::UART2.start()),
            uart3: PhysicalInstance::new(*MemoryMap::UART3.start()),
            watchdog: PhysicalInstance::new(*MemoryMap::WATCHDOG.start()),
            power_controller: PhysicalInstance::new(*MemoryMap::POWER_CONTROLLER.start()),
            cci_550: PhysicalInstance::new(*MemoryMap::CCI_550.start()),
            refclk_cntcontrol: PhysicalInstance::new(*MemoryMap::REFCLK_CNTCONTROL.start()),
            trusted_watchdog: PhysicalInstance::new(*MemoryMap::TRUSTED_WATCHDOG.start()),
            refclk_cntread: PhysicalInstance::new(*MemoryMap::REFCLK_CNTREAD.start()),
            ap_refclk_cntctl: PhysicalInstance::new(*MemoryMap::AP_REFCLK_CNTCTL.start()),
            ap_refclk_cntbase0: PhysicalInstance::new(*MemoryMap::AP_REFCLK_CNTBASE0.start()),
            ap_refclk_cntbase1: PhysicalInstance::new(*MemoryMap::AP_REFCLK_CNTBASE1.start()),
            gicd: PhysicalInstance::new(*MemoryMap::GICD.start()),
            gicr: PhysicalInstance::new(*MemoryMap::GICR.start()),
        }
    }
}

/// CCI-550 index assignment of internal components.
pub struct Cci550Map;

impl Cci550Map {
    /// Index of cluster 0.
    pub const CLUSTER0: usize = 5;
    /// Index of cluster 1.
    pub const CLUSTER1: usize = 6;
}
