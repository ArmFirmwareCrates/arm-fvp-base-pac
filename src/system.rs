// SPDX-FileCopyrightText: Copyright The arm-fvp-base-pac Contributors.
// SPDX-License-Identifier: MIT OR Apache-2.0

//! FVP System Peripheral driver implementation.

use bitflags::bitflags;
use safe_mmio::{
    UniqueMmioPointer, field, field_shared,
    fields::{ReadPure, ReadPureWrite, WriteOnly},
};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

/// Status of data transfer to motherboard controller.
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
struct SysCfgStat(u32);

bitflags! {
    impl SysCfgStat: u32 {
        const ERROR = 1 << 1;
        const COMPLETE = 1 << 0;
    }
}

/// FVP System Peripheral registers based on 'Table 5-17 VE_SysRegs registers'
#[derive(Clone, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(C, align(4))]
pub struct FvpSystemRegisters {
    /// 0x00 System identity
    sys_id: ReadPure<u32>,
    /// 0x04 Bits\[7:0\] map to switch S6.
    sys_sw: ReadPureWrite<u32>,
    /// 0x08 Bits\[7:0\] map to user LEDs.
    sys_led: ReadPureWrite<u32>,
    /// 0x0c - 0x20
    reserved_0c: [u32; 6],
    /// 0x24 100Hz counter.
    sys_100hz: ReadPure<u32>,
    /// 0x28 - 0x2c
    reserved_28: [u32; 2],
    /// 0x30 General-purpose flags.
    sys_flags: ReadPureWrite<u32>,
    /// 0x34 Clear bits in general-purpose flags.
    sys_flagsclr: WriteOnly<u32>,
    /// 0x38 General-purpose non-volatile flags.
    sys_nvflags: ReadPureWrite<u32>,
    /// 0x3c Clear bits in general-purpose non-volatile flags.
    sys_nvflagsclr: WriteOnly<u32>,
    /// 0x40 - 0x44
    reserved_40: [u32; 2],
    /// 0x48 MCI.
    sys_mci: ReadPure<u32>,
    /// 0x4c Flash control.
    sys_flash: ReadPureWrite<u32>,
    /// 0x50 - 0x54
    reserved_50: [u32; 2],
    /// 0x58 Boot-select switch.
    sys_cfgsw: ReadPureWrite<u32>,
    /// 0x5c 24MHz counter.
    sys_24mhz: ReadPure<u32>,
    /// 0x60 Miscellaneous control flags.
    sys_misc: ReadPureWrite<u32>,
    /// 0x64 Read/write DMA peripheral map.
    sys_dma: ReadPureWrite<u32>,
    /// 0x68 - 0x80
    reserved_68: [u32; 7],
    /// 0x84 Processor ID.
    sys_procid0: ReadPureWrite<u32>,
    /// 0x88 Processor ID.
    sys_procid1: ReadPureWrite<u32>,
    /// 0x8c Processor ID.
    sys_procid2: ReadPureWrite<u32>,
    /// 0x90 Processor ID.
    sys_procid3: ReadPureWrite<u32>,
    /// 0x94 - 0x9c
    reserved_8c: [u32; 3],
    /// 0xa0 Data to read/write from & to motherboard controller.
    sys_cfgdata: ReadPureWrite<u32>,
    /// 0xa4 Control data transfer to motherboard controller.
    sys_cfgctrl: ReadPureWrite<u32>,
    /// 0xa8 Status of data transfer to motherboard controller.
    sys_cfgstat: ReadPure<SysCfgStat>,
    /// 0xac - 0xffc
    reserved_ac: [u32; 981],
}

/// FVP System Peripheral error type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    InvalidBoardRevision,
    InvalidHbi,
    InvalidVariant,
    InvalidPlatformType,
}

/// Board revision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardRevision {
    RevA,
    RevB,
    RevC,
}

impl TryFrom<u32> for BoardRevision {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            0x0 => Self::RevA,
            0x1 => Self::RevB,
            0x2 => Self::RevC,
            _ => return Err(Error::InvalidBoardRevision),
        })
    }
}

/// HBI platform description.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hbi {
    V8FoundationPlatform,
    V8BasePlatform,
}

impl TryFrom<u32> for Hbi {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            0x10 => Self::V8FoundationPlatform,
            0x20 => Self::V8BasePlatform,
            _ => return Err(Error::InvalidHbi),
        })
    }
}

/// Platform variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variant {
    VariantA = 0x0,
    VariantB = 0x1,
    VariantC = 0x2,
}

impl TryFrom<u32> for Variant {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            0x0 => Self::VariantA,
            0x1 => Self::VariantB,
            0x2 => Self::VariantC,
            _ => return Err(Error::InvalidVariant),
        })
    }
}

/// Platform type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformType {
    Board,
    Model,
    Emulator,
    Simulator,
    Fpga,
}

impl TryFrom<u32> for PlatformType {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            0x0 => Self::Board,
            0x1 => Self::Model,
            0x2 => Self::Emulator,
            0x3 => Self::Simulator,
            0x4 => Self::Fpga,
            _ => return Err(Error::InvalidPlatformType),
        })
    }
}

/// System identification of the FVP platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SystemId {
    pub revision: BoardRevision,
    pub hbi: Hbi,
    pub variant: Variant,
    pub platform_type: PlatformType,
    pub fpga_build: u8,
}

impl TryFrom<u32> for SystemId {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(Self {
            revision: ((value >> 28) & 0x0f).try_into()?,
            hbi: ((value >> 16) & 0x3ff).try_into()?,
            variant: ((value >> 12) & 0x0f).try_into()?,
            platform_type: ((value >> 8) & 0x0f).try_into()?,
            fpga_build: value as u8,
        })
    }
}

/// System configuration functions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum SystemConfigFunction {
    /// Shutdown system.
    Shutdown = 0x08,
    /// Reboot system.
    Reboot = 0x09,
}

/// FVP System Peripheral Driver
pub struct FvpSystemPeripheral<'a> {
    regs: UniqueMmioPointer<'a, FvpSystemRegisters>,
}

impl<'a> FvpSystemPeripheral<'a> {
    const SYSCFG_START: u32 = 1 << 31;
    const SYSCFG_WRITE: u32 = 1 << 30;
    const SYSCFG_FUNC_SHIFT: usize = 20;

    /// Create new driver instance.
    pub fn new(regs: UniqueMmioPointer<'a, FvpSystemRegisters>) -> Self {
        Self { regs }
    }

    /// Reads system ID.
    pub fn system_id(&self) -> Result<SystemId, Error> {
        field_shared!(self.regs, sys_id).read().try_into()
    }

    /// Reads switch state.
    pub fn switches(&self) -> u8 {
        field_shared!(self.regs, sys_sw).read() as u8
    }

    /// Sets LED state.
    pub fn set_leds(&mut self, leds: u8) {
        field!(self.regs, sys_led).write(leds as u32);
    }

    /// Reads LED state.
    pub fn leds(&self) -> u8 {
        field_shared!(self.regs, sys_led).read() as u8
    }

    /// Reads 100Hz counter value.
    pub fn counter_100hz(&self) -> u32 {
        field_shared!(self.regs, sys_100hz).read()
    }

    /// Sets flag bits.
    pub fn set_flags(&mut self, flags: u32) {
        field!(self.regs, sys_flags).write(flags);
    }

    /// Clears flag bits.
    pub fn clear_flags(&mut self, flags: u32) {
        field!(self.regs, sys_flagsclr).write(flags);
    }

    /// Reads flag bits.
    pub fn flags(&self) -> u32 {
        field_shared!(self.regs, sys_flags).read()
    }

    /// Sets non-volatile flag bits.
    pub fn set_non_volatile_flags(&mut self, flags: u32) {
        field!(self.regs, sys_nvflags).write(flags);
    }

    /// Clears non-volatile flag bits.
    pub fn clear_non_volatile_flags(&mut self, flags: u32) {
        field!(self.regs, sys_nvflagsclr).write(flags);
    }

    /// Reads non-volatile flag bits.
    pub fn non_volatile_flags(&self) -> u32 {
        field_shared!(self.regs, sys_nvflags).read()
    }

    /// Checks that the MMC card is present.
    pub fn mmc_card_present(&self) -> bool {
        (field_shared!(self.regs, sys_mci).read() & 0x0000_0001) != 0
    }

    /// Sets flash control register value.
    pub fn set_flash_control(&mut self, value: u32) {
        field!(self.regs, sys_flash).write(value);
    }

    /// Reads flash control register value.
    pub fn flash_control(&self) -> u32 {
        field_shared!(self.regs, sys_flash).read()
    }

    /// Read boot select switch.
    pub fn boot_select_switch(&self) -> u32 {
        field_shared!(self.regs, sys_cfgsw).read()
    }

    /// Read 24MHz counter.
    pub fn counter_24mhz(&self) -> u32 {
        field_shared!(self.regs, sys_24mhz).read()
    }

    /// Miscellaneous control flags.
    pub fn misc(&self) -> u32 {
        field_shared!(self.regs, sys_misc).read()
    }

    /// Reads DMA peripheral map.
    pub fn dma(&self) -> u32 {
        field_shared!(self.regs, sys_dma).read()
    }

    /// Reads processor ID.
    pub fn processor_id(&self) -> [u32; 4] {
        [
            field_shared!(self.regs, sys_procid0).read(),
            field_shared!(self.regs, sys_procid1).read(),
            field_shared!(self.regs, sys_procid2).read(),
            field_shared!(self.regs, sys_procid3).read(),
        ]
    }

    /// Writes system configuration and its optional data.
    pub fn write_system_configuration(&mut self, function: SystemConfigFunction) {
        field!(self.regs, sys_cfgctrl).write(
            Self::SYSCFG_START
                | Self::SYSCFG_WRITE
                | ((function as u32) << Self::SYSCFG_FUNC_SHIFT),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zerocopy::transmute_mut;

    #[repr(align(4096))]
    pub struct FakeSystemRegisters {
        regs: [u32; 1024],
    }

    impl FakeSystemRegisters {
        pub fn new() -> Self {
            Self { regs: [0u32; 1024] }
        }

        pub fn clear(&mut self) {
            self.regs.fill(0);
        }

        pub fn reg_write(&mut self, offset: usize, value: u32) {
            self.regs[offset / 4] = value;
        }

        pub fn reg_read(&self, offset: usize) -> u32 {
            self.regs[offset / 4]
        }

        fn get(&mut self) -> UniqueMmioPointer<'_, FvpSystemRegisters> {
            UniqueMmioPointer::from(transmute_mut!(&mut self.regs))
        }

        pub fn system_for_test(&mut self) -> FvpSystemPeripheral<'_> {
            FvpSystemPeripheral::new(self.get())
        }
    }

    #[test]
    fn regs_size() {
        assert_eq!(core::mem::size_of::<FvpSystemRegisters>(), 0x1000);
    }

    #[test]
    fn board_revision() {
        assert_eq!(Ok(BoardRevision::RevA), BoardRevision::try_from(0x0));
        assert_eq!(Ok(BoardRevision::RevB), BoardRevision::try_from(0x1));
        assert_eq!(Ok(BoardRevision::RevC), BoardRevision::try_from(0x2));
        assert_eq!(
            Err(Error::InvalidBoardRevision),
            BoardRevision::try_from(0x3)
        );
    }

    #[test]
    fn hbi() {
        assert_eq!(Ok(Hbi::V8FoundationPlatform), Hbi::try_from(0x10));
        assert_eq!(Ok(Hbi::V8BasePlatform), Hbi::try_from(0x20));
        assert_eq!(Err(Error::InvalidHbi), Hbi::try_from(0xff));
    }

    #[test]
    fn variant() {
        assert_eq!(Ok(Variant::VariantA), Variant::try_from(0x0));
        assert_eq!(Ok(Variant::VariantB), Variant::try_from(0x1));
        assert_eq!(Ok(Variant::VariantC), Variant::try_from(0x2));
        assert_eq!(Err(Error::InvalidVariant), Variant::try_from(0x3));
    }

    #[test]
    fn platform_type() {
        assert_eq!(Ok(PlatformType::Board), PlatformType::try_from(0x0));
        assert_eq!(Ok(PlatformType::Model), PlatformType::try_from(0x1));
        assert_eq!(Ok(PlatformType::Emulator), PlatformType::try_from(0x2));
        assert_eq!(Ok(PlatformType::Simulator), PlatformType::try_from(0x3));
        assert_eq!(Ok(PlatformType::Fpga), PlatformType::try_from(0x4));
        assert_eq!(Err(Error::InvalidPlatformType), PlatformType::try_from(0x5));
    }

    #[test]
    fn system_id() {
        assert_eq!(
            Ok(SystemId {
                revision: BoardRevision::RevC,
                hbi: Hbi::V8BasePlatform,
                variant: Variant::VariantC,
                platform_type: PlatformType::Fpga,
                fpga_build: 0x5a
            }),
            SystemId::try_from(0x2020_245a)
        );

        assert_eq!(
            Err(Error::InvalidBoardRevision),
            SystemId::try_from(0xf020_245a)
        );
        assert_eq!(Err(Error::InvalidHbi), SystemId::try_from(0x20f0_245a));
        assert_eq!(Err(Error::InvalidVariant), SystemId::try_from(0x2020_f45a));
        assert_eq!(
            Err(Error::InvalidPlatformType),
            SystemId::try_from(0x2020_2f5a)
        );
    }

    #[test]
    fn system_peripheral() {
        let mut regs = FakeSystemRegisters::new();

        {
            // System ID
            regs.reg_write(0x00, 0x2020_245a);
            let sys = regs.system_for_test();
            assert_eq!(
                Ok(SystemId {
                    revision: BoardRevision::RevC,
                    hbi: Hbi::V8BasePlatform,
                    variant: Variant::VariantC,
                    platform_type: PlatformType::Fpga,
                    fpga_build: 0x5a
                }),
                sys.system_id()
            );
        }

        regs.clear();

        {
            // Switches
            regs.reg_write(0x04, 0xabcd_ef5a);
            let sys = regs.system_for_test();
            assert_eq!(0x5a, sys.switches());
        }

        regs.clear();

        {
            // LEDs
            regs.reg_write(0x08, 0xabcd_ef5a);
            let mut sys = regs.system_for_test();
            assert_eq!(0x5a, sys.leds());
            sys.set_leds(0xa5);
        }

        assert_eq!(0xa5, regs.reg_read(0x08));

        regs.clear();

        {
            // 100Hz counter
            regs.reg_write(0x24, 100);
            let sys = regs.system_for_test();
            assert_eq!(100, sys.counter_100hz());
        }

        regs.clear();

        {
            // Flags
            let mut sys = regs.system_for_test();
            // The set and read registers are the same, the clear register is a separate one.
            sys.set_flags(0x0123_4567);
            assert_eq!(0x0123_4567, sys.flags());
            sys.clear_flags(0x89ab_cdef);
        }

        assert_eq!(0x0123_4567, regs.reg_read(0x30));
        assert_eq!(0x89ab_cdef, regs.reg_read(0x34));
        regs.clear();

        {
            // Non-volatile flags
            let mut sys = regs.system_for_test();
            // The set and read registers are the same, the clear register is a separate one.
            sys.set_non_volatile_flags(0x0123_4567);
            assert_eq!(0x0123_4567, sys.non_volatile_flags());
            sys.clear_non_volatile_flags(0x89ab_cdef);
        }

        assert_eq!(0x0123_4567, regs.reg_read(0x38));
        assert_eq!(0x89ab_cdef, regs.reg_read(0x3c));
        regs.clear();

        {
            // MMC card present
            regs.reg_write(0x48, 0x0000_0001);
            let sys = regs.system_for_test();
            assert!(sys.mmc_card_present());
        }

        regs.clear();

        {
            // Flash control
            let mut sys = regs.system_for_test();
            sys.set_flash_control(0x89ab_cdef);
            assert_eq!(0x89ab_cdef, sys.flash_control());
        }

        assert_eq!(0x89ab_cdef, regs.reg_read(0x4c));
        regs.clear();

        {
            // Boot select switch
            regs.reg_write(0x58, 0x89ab_cdef);
            let sys = regs.system_for_test();
            assert_eq!(0x89ab_cdef, sys.boot_select_switch());
        }

        regs.clear();

        {
            // 24MHz counter
            regs.reg_write(0x5c, 0x89ab_cdef);
            let sys = regs.system_for_test();
            assert_eq!(0x89ab_cdef, sys.counter_24mhz());
        }

        regs.clear();

        {
            // Miscellaneous control flags.
            regs.reg_write(0x60, 0x89ab_cdef);
            let sys = regs.system_for_test();
            assert_eq!(0x89ab_cdef, sys.misc());
        }

        regs.clear();

        {
            // DMA peripheral map
            regs.reg_write(0x64, 0x89ab_cdef);
            let sys = regs.system_for_test();
            assert_eq!(0x89ab_cdef, sys.dma());
        }

        regs.clear();

        {
            // Processor ID
            regs.reg_write(0x84, 0x0123_4567);
            regs.reg_write(0x88, 0x89ab_cdef);
            regs.reg_write(0x8c, 0x4567_89ab);
            regs.reg_write(0x90, 0xcdef_0123);
            let sys = regs.system_for_test();

            assert_eq!(
                [0x0123_4567, 0x89ab_cdef, 0x4567_89ab, 0xcdef_0123],
                sys.processor_id()
            );
        }

        regs.clear();

        {
            // System configuration
            let mut sys = regs.system_for_test();
            sys.write_system_configuration(SystemConfigFunction::Shutdown);
        }

        assert_eq!(0xc080_0000, regs.reg_read(0xa4));
    }
}
