// SPDX-FileCopyrightText: Copyright 2025 Arm Limited and/or its affiliates <open-source-office@arm.com>
// SPDX-License-Identifier: MIT OR Apache-2.0

//! FVP Power Controller driver.

use bitflags::bitflags;
use safe_mmio::UniqueMmioPointer;
use safe_mmio::{field, fields::ReadPureWrite};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

/// Power on reason.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PowerOnReason {
    /// Cold power-on.
    ColdPowerOn,
    /// System reset pin.
    SystemResetPin,
    /// Wake by PPONR.
    WakeByProcessorOn,
    /// Wake by GIC WakeRequest signal.
    WakeByGicSignal,
}

impl PowerOnReason {
    const SHIFT: u32 = 24;
    const MASK: u32 = 0b11;
    const COLD_PWR_ON: u32 = 0b00;
    const SYS_RESET_PIN: u32 = 0b01;
    const BY_PPONR: u32 = 0b10;
    const BY_GIC_WAKE_REQ_SIGNAL: u32 = 0b11;
}

impl From<u32> for PowerOnReason {
    fn from(value: u32) -> Self {
        let masked_shifted_value = (value >> Self::SHIFT) & Self::MASK;

        match masked_shifted_value {
            Self::COLD_PWR_ON => PowerOnReason::ColdPowerOn,
            Self::SYS_RESET_PIN => PowerOnReason::SystemResetPin,
            Self::BY_PPONR => PowerOnReason::WakeByProcessorOn,
            Self::BY_GIC_WAKE_REQ_SIGNAL => PowerOnReason::WakeByGicSignal,
            _ => unreachable!(),
        }
    }
}

bitflags! {
    /// Power Control Wakeup Register
    struct WakeupRegister: u32 {
        /// If set, enables wakeup interrupts (return from SUSPEND) for this cluster.
        const WEN = 1 << 31;
    }

    /// Power Control SYS Status Register
    pub struct SystemStatus: u32 {
        /// A value of 1 indicates that affinity level 2 is active/on. If affinity level 2 is not
        /// implemented this bit is RAZ.
        const L2 = 1 << 31;
        /// A value of 1 indicates that affinity level 1 is active/on. If affinity level 1 is not
        /// implemented this bit is RAZ.
        const L1 = 1 << 30;
        /// A value of 1 indicates that affinity level 0 is active/on.
        const L0 = 1 << 29;
        /// A value of 1 indicates wakeup interrupts, return from SUSPEND, enabled for this
        /// processor. This is an alias of PWKUPR.WEN for this core.
        const WEN = 1 << 28;
        /// A value of 1 indicates pending cluster off, the cluster enters low-power mode the next
        /// time it raises signal STANDBYWFIL2.
        const PC = 1 << 27;
        /// A value of 1 indicates pending processor off, the processor enters low-power mode the
        /// next time it raises signal STANDBYWFI.
        const PP = 1 << 26;
    }
}

/// FVP Power Controller register map
#[derive(Clone, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(C, align(4))]
pub struct FvpPowerControllerRegisters {
    /// 0x00 - Power Control Processor Off Register
    ppoffr: ReadPureWrite<u32>,
    /// 0x04 - Power Control Processor On Register
    pponr: ReadPureWrite<u32>,
    /// 0x08 - Power Control Cluster Off Register
    pcoffr: ReadPureWrite<u32>,
    /// 0x0C - Power Control Wakeup Register
    pwkupr: ReadPureWrite<u32>,
    /// 0x10 - Power Control SYS Status Register
    psysr: ReadPureWrite<u32>,
}

/// FVP Power Controller implementation
pub struct FvpPowerController<'a> {
    regs: UniqueMmioPointer<'a, FvpPowerControllerRegisters>,
}

impl<'a> FvpPowerController<'a> {
    const MPIDR_MASK: u32 = 0xff_ffff;

    /// Creates new FVP Power Controller instance.
    pub fn new(regs: UniqueMmioPointer<'a, FvpPowerControllerRegisters>) -> Self {
        Self { regs }
    }

    // Provides the value of the PSYS register as a u32 for internal use only.
    fn system_status_reg(&mut self, mpidr: u32) -> u32 {
        field!(self.regs, psysr).write(mpidr & Self::MPIDR_MASK);
        field!(self.regs, psysr).read()
    }

    /// Provides information on the powered status of a given core.
    ///
    /// This is done by writing the ID for the required core to the PSYS register and then reading
    /// the value along with the associated status.
    /// Please see `power_on_reason` for other related information.
    pub fn system_status(&mut self, mpidr: u32) -> SystemStatus {
        // There are no usage constraints
        SystemStatus::from_bits_truncate(self.system_status_reg(mpidr))
    }

    /// Brings up the given processor from low-power mode by writing to the PPONR register
    ///
    /// Processor must make power-on requests only for other powered-off processors in the system,
    /// otherwise get a Programming error.
    pub fn power_on_processor(&mut self, mpidr: u32) {
        field!(self.regs, pponr).write(mpidr & Self::MPIDR_MASK);
    }

    /// Processor SUSPEND command (by writing to the PPOFFR register)
    ///
    /// when PWKUPR and the GIC are programmed appropriately to provide wakeup events from IRQ and
    /// FIQ events to that processor.
    /// Processor must make power-off requests only for itself, otherwise get a Programming error.
    pub fn power_off_processor(&mut self, mpidr: u32) {
        field!(self.regs, ppoffr).write(mpidr & Self::MPIDR_MASK);
    }

    /// Turns the cluster off
    ///
    /// Cluster must make power-off requests only for itself, otherwise get a Programming error.
    pub fn power_off_cluster(&mut self, mpidr: u32) {
        field!(self.regs, pcoffr).write(mpidr & Self::MPIDR_MASK);
    }

    /// Configures whether wakeup requests from the GIC are enabled for this cluster
    pub fn disable_wakeup_requests(&mut self, mpidr: u32) {
        // There are no usage constraints
        let wkup_reg = WakeupRegister::empty();

        field!(self.regs, pwkupr).write((mpidr & Self::MPIDR_MASK) | wkup_reg.bits());
    }

    /// Configures whether wakeup requests from the GIC are enabled for this cluster
    pub fn enable_wakeup_requests(&mut self, mpidr: u32) {
        // There are no usage constraints
        let wkup_reg = WakeupRegister::empty().union(WakeupRegister::WEN);

        field!(self.regs, pwkupr).write((mpidr & Self::MPIDR_MASK) | wkup_reg.bits());
    }

    /// Provides information on the reason for Power On of the given core.
    ///
    /// This is done by writing the ID for the required core to the PSYS register and reading the
    /// value along with the associated status.
    /// Please see `system_status` for other related information.
    pub fn power_on_reason(&mut self, mpidr: u32) -> PowerOnReason {
        PowerOnReason::from(self.system_status_reg(mpidr))
    }
}

// SAFETY: An `&FvpPowerController` only allows operations which read registers, which can safely be
// done from multiple threads simultaneously.
unsafe impl Sync for FvpPowerController<'_> {}

#[cfg(test)]
mod tests {
    use super::*;
    use zerocopy::transmute_mut;

    pub struct FakeFvpPowerControllerRegisters {
        regs: [u32; 5],
    }

    impl FakeFvpPowerControllerRegisters {
        pub fn new() -> Self {
            Self { regs: [0u32; 5] }
        }

        pub fn reg_read(&self, offset: usize) -> u32 {
            self.regs[offset / 4]
        }

        fn get(&mut self) -> UniqueMmioPointer<FvpPowerControllerRegisters> {
            UniqueMmioPointer::from(transmute_mut!(&mut self.regs))
        }

        pub fn fvp_power_controller_for_test(&mut self) -> FvpPowerController {
            FvpPowerController::new(self.get())
        }
    }

    #[test]
    fn regs_size() {
        assert_eq!(core::mem::size_of::<FvpPowerControllerRegisters>(), 0x14);
    }

    #[test]
    fn sys_status() {
        let mut regs = FakeFvpPowerControllerRegisters::new();
        let fake_mpidr = 111234;
        let mut fvp_power_controller = regs.fvp_power_controller_for_test();

        let sys_status = fvp_power_controller.system_status(fake_mpidr);

        assert!(!sys_status.contains(SystemStatus::L2));
        assert!(!sys_status.contains(SystemStatus::L1));
        assert!(!sys_status.contains(SystemStatus::L0));
        assert!(!sys_status.contains(SystemStatus::WEN));
        assert!(!sys_status.contains(SystemStatus::PC));
        assert!(!sys_status.contains(SystemStatus::PP));
    }

    #[test]
    fn pwkupr() {
        let mut regs = FakeFvpPowerControllerRegisters::new();
        let fake_mpidr = 865032;
        let wen_flag = 1 << 31;
        {
            let mut fvp_power_controller = regs.fvp_power_controller_for_test();
            fvp_power_controller.enable_wakeup_requests(fake_mpidr);

            assert_eq!(regs.reg_read(0x0C), fake_mpidr | wen_flag);
        }

        {
            let mut fvp_power_controller = regs.fvp_power_controller_for_test();
            fvp_power_controller.disable_wakeup_requests(fake_mpidr);

            assert_eq!(regs.reg_read(0x0C), fake_mpidr);
        }
    }

    #[test]
    fn pponr() {
        let mut regs = FakeFvpPowerControllerRegisters::new();
        let fake_mpidr = 865032;
        let mut fvp_power_controller = regs.fvp_power_controller_for_test();
        fvp_power_controller.power_off_processor(fake_mpidr);
        assert_eq!(regs.reg_read(0x00), fake_mpidr);
    }

    #[test]
    fn ppoffr() {
        let mut regs = FakeFvpPowerControllerRegisters::new();
        let fake_mpidr = 865032;
        let mut fvp_power_controller = regs.fvp_power_controller_for_test();
        fvp_power_controller.power_on_processor(fake_mpidr);
        assert_eq!(regs.reg_read(0x04), fake_mpidr);
    }

    #[test]
    fn pcoffr() {
        let mut regs = FakeFvpPowerControllerRegisters::new();
        let fake_mpidr = 0b1010_0010_1010_0010_1010_0010;
        let mut fvp_power_controller = regs.fvp_power_controller_for_test();
        fvp_power_controller.power_off_cluster(fake_mpidr);
        assert_eq!(regs.reg_read(0x08), fake_mpidr);
    }

    #[test]
    fn power_on_reason() {
        let mut regs = FakeFvpPowerControllerRegisters::new();
        let fake_mpidr = 0b1010_0010_1010_0010_1010_0010;
        let mut fvp_power_controller = regs.fvp_power_controller_for_test();
        assert_eq!(
            fvp_power_controller.power_on_reason(fake_mpidr),
            PowerOnReason::ColdPowerOn
        );
    }

    #[test]
    fn power_on_reason_enum() {
        // Power on Reason bits: 00 -> Cold power-on.
        let sysr_cold_pwr_on = 0b1011_1100_1010_0010_1010_0010_1010_0010;
        // Power on Reason bits: 11 -> Wake by GIC WakeRequest signal.
        let sysr_gic_wake_req_sig = !sysr_cold_pwr_on;
        // Power on Reason bits: 01 -> System reset pin.
        let sysr_system_reset_pin = 0b0110_1001_1010_0110_1111_0000_1010_0010;
        // Power on Reason bits: 10 -> Wake by PPONR.
        let sysr_by_pponr = !sysr_system_reset_pin;

        assert_eq!(
            PowerOnReason::from(sysr_cold_pwr_on),
            PowerOnReason::ColdPowerOn
        );
        assert_eq!(
            PowerOnReason::from(sysr_gic_wake_req_sig),
            PowerOnReason::WakeByGicSignal
        );
        assert_eq!(
            PowerOnReason::from(sysr_system_reset_pin),
            PowerOnReason::SystemResetPin
        );
        assert_eq!(
            PowerOnReason::from(sysr_by_pponr),
            PowerOnReason::WakeByProcessorOn
        );
    }
}
