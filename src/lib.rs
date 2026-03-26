// SPDX-FileCopyrightText: Copyright The arm-fvp-base-pac Contributors.
// SPDX-License-Identifier: MIT OR Apache-2.0

#![no_std]
#![doc = include_str!("../README.md")]
#![deny(clippy::undocumented_unsafe_blocks)]

#[cfg(all(feature = "base-revc", feature = "base-r"))]
compile_error!("must only choose one of these Cargo features: `base-revc`; `base-r`");

pub mod power_controller;
pub mod system;

// Re-export peripheral drivers and common safe-mmio types
#[cfg(feature = "base-revc")]
pub use arm_cci;
pub use arm_generic_timer;
pub use arm_gic;
pub use arm_pl011_uart;
pub use arm_sp805;
pub use arm_tzc;

pub use safe_mmio::{PhysicalInstance, UniqueMmioPointer};

#[cfg(feature = "base-revc")]
use arm_cci::Cci5x0Registers;
use arm_generic_timer::{CntBase, CntControlBase, CntCtlBase, CntReadBase};
use arm_gic::{
    gicv3::registers::{Gicd, GicrSgi},
    IntId,
};
use arm_pl011_uart::PL011Registers;
use arm_sp805::SP805Registers;
use arm_tzc::TzcRegisters;
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
    pub const TRUSTED_BOOT_ROM: RangeInclusive<usize> = flip(0x00_0000_0000..=0x00_03FF_FFFF);
    pub const TRUSTED_SRAM: RangeInclusive<usize> = flip(0x00_0400_0000..=0x00_0407_FFFF);
    pub const TRUSTED_DRAM: RangeInclusive<usize> = flip(0x00_0600_0000..=0x00_07FF_FFFF);
    pub const NOR_FLASH0: RangeInclusive<usize> = flip(0x00_0800_0000..=0x00_0BFF_FFFF);
    pub const NOR_FLASH1: RangeInclusive<usize> = flip(0x00_0C00_0000..=0x00_0FFF_FFFF);
    pub const PSRAM: RangeInclusive<usize> = flip(0x00_1400_0000..=0x00_17FF_FFFF);
    pub const VRAM: RangeInclusive<usize> = flip(0x00_1800_0000..=0x00_19FF_FFFF);
    pub const ETHERNET: RangeInclusive<usize> = flip(0x00_1A00_0000..=0x00_1AFF_FFFF);
    pub const USB: RangeInclusive<usize> = flip(0x00_1B00_0000..=0x00_1BFF_FFFF);
    pub const VE_SYSTEM: RangeInclusive<usize> = flip(0x00_1C01_0000..=0x00_1C01_FFFF);
    pub const SYSTEM_CONTROLLER: RangeInclusive<usize> = flip(0x00_1C02_0000..=0x00_1C02_FFFF);
    pub const AACI: RangeInclusive<usize> = flip(0x00_1C04_0000..=0x00_1C04_FFFF);
    pub const MCI: RangeInclusive<usize> = flip(0x00_1C05_0000..=0x00_1C05_FFFF);
    pub const KMI_KEYBOARD: RangeInclusive<usize> = flip(0x00_1C06_0000..=0x00_1C06_FFFF);
    pub const KMI_MOUSE: RangeInclusive<usize> = flip(0x00_1C07_0000..=0x00_1C07_FFFF);
    pub const UART0: RangeInclusive<usize> = flip(0x00_1C09_0000..=0x00_1C09_FFFF);
    pub const UART1: RangeInclusive<usize> = flip(0x00_1C0A_0000..=0x00_1C0A_FFFF);
    pub const UART2: RangeInclusive<usize> = flip(0x00_1C0B_0000..=0x00_1C0B_FFFF);
    pub const UART3: RangeInclusive<usize> = flip(0x00_1C0C_0000..=0x00_1C0C_FFFF);
    pub const VFS2: RangeInclusive<usize> = flip(0x00_1C0D_0000..=0x00_1C0D_FFFF);
    pub const WATCHDOG: RangeInclusive<usize> = flip(0x00_1C0F_0000..=0x00_1C0F_FFFF);
    pub const POWER_CONTROLLER: RangeInclusive<usize> = flip(0x00_1C10_0000..=0x00_1C10_FFFF);
    pub const DUAL_TIMER0: RangeInclusive<usize> = flip(0x00_1C11_0000..=0x00_1C11_FFFF);
    pub const DUAL_TIMER1: RangeInclusive<usize> = flip(0x00_1C12_0000..=0x00_1C12_FFFF);
    pub const VIRTIO_BLOCK_DEVICE: RangeInclusive<usize> = flip(0x00_1C13_0000..=0x00_1C13_FFFF);
    pub const VIRTIO_PLAN9_DEVICE: RangeInclusive<usize> = flip(0x00_1C14_0000..=0x00_1C14_FFFF);
    pub const VIRTIO_NET_DEVICE: RangeInclusive<usize> = flip(0x00_1C15_0000..=0x00_1C15_FFFF);
    pub const RTC: RangeInclusive<usize> = flip(0x00_1C17_0000..=0x00_1C17_FFFF);
    pub const CF_CARD: RangeInclusive<usize> = flip(0x00_1C1A_0000..=0x00_1C1A_FFFF);
    pub const CLCD_CONTROLLER: RangeInclusive<usize> = flip(0x00_1C1F_0000..=0x00_1C1F_FFFF);
    pub const VIRTIO_RNG: RangeInclusive<usize> = flip(0x00_1C20_0000..=0x00_1C20_FFFF);
    pub const LS64_TESTING_FIFO: RangeInclusive<usize> = flip(0x00_1D00_0000..=0x00_1D00_FFFF);
    pub const UTILITY_BUS: RangeInclusive<usize> = flip(0x00_1E00_0000..=0x00_1EFF_FFFF);
    pub const NON_TRUSTED_ROM: RangeInclusive<usize> = flip(0x00_1F00_0000..=0x00_1F00_0FFF);
    pub const CORESIGHT: RangeInclusive<usize> = flip(0x00_2000_0000..=0x00_27FF_FFFF);
    #[cfg(feature = "base-revc")]
    pub const CCI_550: RangeInclusive<usize> = flip(0x00_2A00_0000..=0x00_2A09_FFFF);
    pub const REFCLK_CNTCONTROL: RangeInclusive<usize> = flip(0x00_2A43_0000..=0x00_2A43_FFFF);
    pub const EL2_WATCHDOG_CONTROL: RangeInclusive<usize> = flip(0x00_2A44_0000..=0x00_2A44_FFFF);
    pub const EL2_WATCHDOG_REFRESH: RangeInclusive<usize> = flip(0x00_2A45_0000..=0x00_2A45_FFFF);
    pub const TRUSTED_WATCHDOG: RangeInclusive<usize> = flip(0x00_2A49_0000..=0x00_2A49_FFFF);
    pub const TRUSTZONE_CONTROLLER: RangeInclusive<usize> = flip(0x00_2A4A_0000..=0x00_2A4A_FFFF);
    pub const REFCLK_CNTREAD: RangeInclusive<usize> = flip(0x00_2A80_0000..=0x00_2A80_FFFF);
    pub const AP_REFCLK_CNTCTL: RangeInclusive<usize> = flip(0x00_2A81_0000..=0x00_2A81_FFFF);
    pub const AP_REFCLK_CNTBASE0: RangeInclusive<usize> = flip(0x00_2A82_0000..=0x00_2A82_FFFF);
    pub const AP_REFCLK_CNTBASE1: RangeInclusive<usize> = flip(0x00_2A83_0000..=0x00_2A83_FFFF);
    pub const DMC_400_CFG: RangeInclusive<usize> = flip(0x00_2B0A_0000..=0x00_2B0A_FFFF);
    #[cfg(feature = "base-revc")]
    pub const SMMUV3_AEM: RangeInclusive<usize> = flip(0x00_2B40_0000..=0x00_2B4F_FFFF);
    #[cfg(feature = "base-revc")]
    pub const DMA330X4: RangeInclusive<usize> = flip(0x00_2B50_0000..=0x00_2B5F_FFFF);
    pub const GICC: RangeInclusive<usize> = flip(0x00_2C00_0000..=0x00_2C00_1FFF);
    pub const GICH: RangeInclusive<usize> = flip(0x00_2C01_0000..=0x00_2C01_0FFF);
    pub const GICV: RangeInclusive<usize> = flip(0x00_2C02_F000..=0x00_2C03_0FFF);
    #[cfg(not(feature = "base-revc"))]
    pub const CCI_400: RangeInclusive<usize> = flip(0x00_2C09_0000..=0x00_2C09_FFFF);
    #[cfg(feature = "base-revc")]
    pub const MALI_G76: RangeInclusive<usize> = flip(0x00_2D00_0000..=0x00_2DFF_0000);
    pub const NON_TRUSTED_SRAM: RangeInclusive<usize> = flip(0x00_2E00_0000..=0x00_2E00_FFFF);
    pub const GICD: RangeInclusive<usize> = flip(0x00_2F00_0000..=0x00_2F00_FFFF);
    pub const GITS: RangeInclusive<usize> = flip(0x00_2F02_0000..=0x00_2F03_FFFF);
    pub const GICR: RangeInclusive<usize> = flip(0x00_2F10_0000..=0x00_2F1F_FFFF);
    #[cfg(feature = "base-revc")]
    pub const PCIE_CONFIG_REGION: RangeInclusive<usize> = flip(0x00_4000_0000..=0x00_4FFF_FFFF);
    #[cfg(feature = "base-revc")]
    pub const PCIE_MEMORY_REGION1: RangeInclusive<usize> = flip(0x00_5000_0000..=0x00_5FFF_FFFF);
    pub const TRUSTED_RNG: RangeInclusive<usize> = flip(0x00_7FE6_0000..=0x00_7FE6_0FFF);
    pub const TRUSTED_NV_COUNTERS: RangeInclusive<usize> = flip(0x00_7FE7_0000..=0x00_7FE7_0FFF);
    pub const TRUSTED_ROOT_KEY_STORAGE: RangeInclusive<usize> =
        flip(0x00_7FE8_0000..=0x00_7FE8_0FFF);
    pub const DDR3_PHY: RangeInclusive<usize> = flip(0x00_7FEF_0000..=0x00_7FEF_FFFF);
    pub const HDLCD_CONTROLLER: RangeInclusive<usize> = flip(0x00_7FF6_0000..=0x00_7FF6_FFFF);
    pub const DRAM0: RangeInclusive<usize> = flip(0x00_8000_0000..=0x00_FFFF_FFFF);
    pub const DRAM1: RangeInclusive<usize> = flip(0x08_8000_0000..=0x0F_FFFF_FFFF);
    #[cfg(feature = "base-revc")]
    pub const PCIE_MEMORY_REGION2: RangeInclusive<usize> = flip(0x40_0000_0000..=0x7F_FFFF_FFFF);
    pub const DRAM2: RangeInclusive<usize> = flip(0x88_0000_0000..=0xFF_FFFF_FFFF);
    pub const DRAM3: RangeInclusive<usize> = flip(0x00_0880_0000_0000..=0x00_0FFF_FFFF_FFFF);
    pub const DRAM4: RangeInclusive<usize> = flip(0x00_8800_0000_0000..=0x00_FFFF_FFFF_FFFF);
    pub const DRAM5: RangeInclusive<usize> = flip(0x08_8000_0000_0000..=0x0F_FFFF_FFFF_FFFF);
    pub const DRAM6: RangeInclusive<usize> = flip(0x88_0000_0000_0000..=0x8F_FFFF_FFFF_FFFF);
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
    #[cfg(feature = "base-revc")]
    pub cci_550: PhysicalInstance<Cci5x0Registers>,
    pub refclk_cntcontrol: PhysicalInstance<CntControlBase>,
    pub trusted_watchdog: PhysicalInstance<SP805Registers>,
    pub trustzone_controller: PhysicalInstance<TzcRegisters>,
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
            #[cfg(feature = "base-revc")]
            cci_550: PhysicalInstance::new(*MemoryMap::CCI_550.start()),
            refclk_cntcontrol: PhysicalInstance::new(*MemoryMap::REFCLK_CNTCONTROL.start()),
            trusted_watchdog: PhysicalInstance::new(*MemoryMap::TRUSTED_WATCHDOG.start()),
            trustzone_controller: PhysicalInstance::new(*MemoryMap::TRUSTZONE_CONTROLLER.start()),
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
#[cfg(feature = "base-revc")]
pub struct Cci550Map;

#[cfg(feature = "base-revc")]
impl Cci550Map {
    /// Index of cluster 0.
    pub const CLUSTER0: usize = 5;
    /// Index of cluster 1.
    pub const CLUSTER1: usize = 6;
}

/// Filter unit assignment in the TrustZone Controller.
pub struct TzcFilter;

impl TzcFilter {
    /// Default filter index.
    pub const DEFAULT: usize = 0;
    /// Filter of PL111_CLCD and HDLCD0.
    pub const LCD: usize = 2;
}

/// Non-Secure Access Identity (NSAID) assignment in the TrustZone Controller.
pub struct TzcNsaid;

impl TzcNsaid {
    /// Default NSAID.
    pub const DEFAULT: usize = 0;
    // Cluster0 and Cluster 1 application processors and VirtioNetMMIO.
    pub const APPLICATION_PROCESSORS: usize = 9;
    /// VirtioP9Device, VirtioBlockDevice.
    pub const VIRTIO: usize = 8;
    /// PCI bus.
    pub const PCI: usize = 1;
    /// PL111_CLCD.
    pub const CLCD: usize = 7;
    /// HDLCD0.
    pub const HDLCD0: usize = 2;
}

/// Private Peripheral Interrupt assignments for the Base Platform.
pub struct PrivatePeripheralInterrupts;

impl PrivatePeripheralInterrupts {
    pub const SECURE_HYPERVISOR_VIRTUAL_TIMER: IntId = IntId::ppi(3);
    pub const SECURE_HYPERVISOR_PHYSICAL_TIMER: IntId = IntId::ppi(4);
    pub const SPU: IntId = IntId::ppi(5);
    pub const DCC_COMMS_CHANNEL: IntId = IntId::ppi(6);
    pub const PMU_OVERFLOW: IntId = IntId::ppi(7);
    pub const CTI: IntId = IntId::ppi(8);
    pub const VIRTUAL_CPU_INTERFACE_MAINTENANCE: IntId = IntId::ppi(9);
    pub const HYPERVISOR_TIMER: IntId = IntId::ppi(10);
    pub const VIRTUAL_TIMER: IntId = IntId::ppi(11);
    pub const HYPERVISOR_VIRTUAL_TIMER: IntId = IntId::ppi(12);
    pub const SECURE_PHYSICAL_TIMER: IntId = IntId::ppi(13);
    pub const NONSECURE_PHYSICAL_TIMER: IntId = IntId::ppi(14);
    pub const TRBU: IntId = IntId::ppi(15);
}

/// Shared Peripheral Interrupt assignments for the Base Platform.
pub struct SharedPeripheralInterrupts;

impl SharedPeripheralInterrupts {
    pub const WATCHDOG: IntId = IntId::spi(0);
    pub const DUAL_TIMER0: IntId = IntId::spi(2);
    pub const DUAL_TIMER1: IntId = IntId::spi(3);
    pub const RTC: IntId = IntId::spi(4);
    pub const UART0: IntId = IntId::spi(5);
    pub const UART1: IntId = IntId::spi(6);
    pub const UART2: IntId = IntId::spi(7);
    pub const UART3: IntId = IntId::spi(8);
    pub const MCIINTR0: IntId = IntId::spi(9);
    pub const MCIINTR1: IntId = IntId::spi(10);
    pub const AACI: IntId = IntId::spi(11);
    pub const KMI_KEYBOARD: IntId = IntId::spi(12);
    pub const KMI_MOUSE: IntId = IntId::spi(13);
    pub const CLCD: IntId = IntId::spi(14);
    pub const CLCD_CONTROLLER: IntId = Self::CLCD;
    pub const ETHERNET: IntId = IntId::spi(15);
    pub const TRUSTED_WATCHDOG: IntId = IntId::spi(24);
    pub const CNTPSIRQ: IntId = IntId::spi(25);
    pub const CNTPSIRQ1: IntId = IntId::spi(26);
    pub const EL2_WATCHDOG_WS0: IntId = IntId::spi(27);
    pub const EL2_WATCHDOG_WS1: IntId = IntId::spi(28);
    pub const VIRTIO_BLOCK_DEVICE: IntId = IntId::spi(42);
    pub const VIRTIO_PLAN9_DEVICE: IntId = IntId::spi(43);
    pub const VIRTIO_NET_DEVICE: IntId = IntId::spi(44);
    pub const VIRTIO_RNG: IntId = IntId::spi(46);
    pub const TRUSTZONE_CONTROLLER: IntId = IntId::spi(48);
    pub const PMUIRQ_CL0_CPU0: IntId = IntId::spi(60);
    pub const PMUIRQ_CL0_CPU1: IntId = IntId::spi(61);
    pub const PMUIRQ_CL0_CPU2: IntId = IntId::spi(62);
    pub const PMUIRQ_CL0_CPU3: IntId = IntId::spi(63);
    pub const PMUIRQ_CL1_CPU0: IntId = IntId::spi(64);
    pub const PMUIRQ_CL1_CPU1: IntId = IntId::spi(65);
    pub const PMUIRQ_CL1_CPU2: IntId = IntId::spi(66);
    pub const PMUIRQ_CL1_CPU3: IntId = IntId::spi(67);
    pub const HDLCD_CONTROLLER: IntId = IntId::spi(85);
    pub const TRUSTED_RNG: IntId = IntId::spi(107);
}

#[cfg(feature = "base-revc")]
impl SharedPeripheralInterrupts {
    pub const SMMUV3_NONSECURE_COMBINED: IntId = IntId::spi(71);
    pub const SMMUV3_SECURE_COMBINED: IntId = IntId::spi(72);
    pub const SMMUV3_SECURE_EVENT_QUEUE: IntId = IntId::spi(73);
    pub const SMMUV3_NONSECURE_EVENT_QUEUE: IntId = IntId::spi(74);
    pub const SMMUV3_PRI_QUEUE: IntId = IntId::spi(75);
    pub const SMMUV3_SECURE_COMMAND_QUEUE_SYNC: IntId = IntId::spi(76);
    pub const SMMUV3_NONSECURE_COMMAND_QUEUE_SYNC: IntId = IntId::spi(77);
    pub const SMMUV3_SECURE_GERROR: IntId = IntId::spi(78);
    pub const SMMUV3_NONSECURE_GERROR: IntId = IntId::spi(79);
    pub const MALI_G76_GPU: IntId = IntId::spi(160);
    pub const MALI_G76_GPU_JOB: IntId = IntId::spi(161);
    pub const MALI_G76_GPU_MMU: IntId = IntId::spi(162);
    pub const PCIE_INTA: IntId = IntId::spi(168);
    pub const PCIE_INTB: IntId = IntId::spi(169);
    pub const PCIE_INTC: IntId = IntId::spi(170);
    pub const PCIE_INTD: IntId = IntId::spi(171);
    pub const PCIE_SERR: IntId = IntId::spi(175);
}

/// For `base-r` platforms, flips the lower 2 GiB and upper 2 GiB regions.
///
/// This is a no-op if the `base-r` feature is not set.
const fn flip(range: RangeInclusive<usize>) -> RangeInclusive<usize> {
    const LIMIT_4GB: usize = 0x1_0000_0000;
    #[cfg(feature = "base-r")]
    const FLIP_BIT: usize = 0x8000_0000;
    #[cfg(not(feature = "base-r"))]
    const FLIP_BIT: usize = 0x0000_0000;

    let mut start = *range.start();
    let mut end = *range.end();
    if start < LIMIT_4GB && end < LIMIT_4GB {
        start ^= FLIP_BIT;
        end ^= FLIP_BIT;
    };
    start..=end
}
