// Copyright (c) 2024, Arm Ltd. All rights reserved.
//
// SPDX-License-Identifier: BSD-3-Clause

use bitflags::bitflags;
use safe_mmio::UniqueMmioPointer;
use safe_mmio::{field, fields::ReadPureWrite};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

// Register descriptions

/// Power Control Processor Off Register
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
struct ProcessorOffRegister(u32);

/// Power Control Processor On Register
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
struct ProcessorOnRegister(u32);

/// Power Control Cluster Off Register
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
struct ClusterOffRegister(u32);

/// Power Control Wakeup Register
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
struct WakeupRegister(u32);

/// Power Control SYS Status Register
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
pub struct SysStatusRegister(u32);

bitflags! {
    impl ProcessorOffRegister: u32 {
    }

    impl ProcessorOnRegister: u32 {
    }

    impl ClusterOffRegister: u32 {

    }

    impl WakeupRegister: u32 {
        /// If set, enables wakeup interrupts (return from SUSPEND) for this cluster.
        const WEN = 1 << 31;
    }

    impl SysStatusRegister: u32 {
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
        /// Cold power-on.
        const WK_COLD_PWR_ON = 0b00 << 25;
        /// System reset pin.
        const WK_SYS_RESET_PIN = 0b01 << 25;
        /// Wake by PPONR.
        const WK_BY_PPONR = 0b10 << 25;
        /// Wake by GIC WakeRequest signal.
        const WK_BY_GIC_WAKE_REQ_SIGNAL = 0b11 << 25;
    }
}

/// FVP Power Controller register map
#[derive(Clone, Eq, FromBytes, Immutable, IntoBytes, PartialEq)]
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
    /// Creates new FVP Power Controller instance.
    pub fn new(regs: UniqueMmioPointer<'a, FvpPowerControllerRegisters>) -> Self {
        Self { regs }
    }

    /// Provides information on the powered status of a given core by writing on the PSYS register
    /// the ID for the required core and reading the value along with the associated status
    pub fn get_sys_status(&mut self, id: u32) -> SysStatusRegister {
        // There are no usage constraints
        field!(self.regs, psysr).write(id);
        SysStatusRegister(field!(self.regs, psysr).read())
    }

    /// Brings up the given processor from low-power mode by writing to the PPONR register
    ///
    /// # Safety
    ///
    /// Processor must make power-on requests only for other powered-off processors in the system.
    pub unsafe fn power_on_proc(&mut self, id: u32) {
        field!(self.regs, pponr).write(id);
    }

    /// Processor SUSPEND command (by writing to the PPOFFR register) when PWKUPR and the GIC are
    /// programmed appropriately to provide wakeup events from IRQ and FIQ events to that processor
    ///
    /// # Safety
    ///
    /// Processor must make power-off requests only for itself.
    pub unsafe fn power_off_proc(&mut self, id: u32) {
        field!(self.regs, ppoffr).write(id);
    }

    /// Turns the cluster off
    ///
    ///  # Safety
    ///
    /// Cluster must make power-off requests only for itself.
    pub unsafe fn power_off_cluster(&mut self, id: u32) {
        field!(self.regs, pcoffr).write(id);
    }

    /// Configures whether wakeup requests from the GIC are enabled for this cluster
    pub fn configure_wakeup_requests(&mut self, id: u32, enable_wkup_ints: bool) {
        // There are no usage constraints

        let mut wkup_reg = WakeupRegister::empty();
        if enable_wkup_ints {
            wkup_reg.insert(WakeupRegister::WEN);
        }

        field!(self.regs, pwkupr).write(id | wkup_reg.bits());
    }
}

// SAFETY: An `&FvpPowerController` only allows operations which read registers, which can safely be done from
// multiple threads simultaneously.
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

        pub fn clear(&mut self) {
            self.regs.fill(0);
        }

        pub fn reg_write(&mut self, offset: usize, value: u32) {
            self.regs[offset / 4] = value;
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
        {
            let mut fvp_power_controller = regs.fvp_power_controller_for_test();
            assert_eq!(fvp_power_controller.get_sys_status(fake_mpidr).bits(), fake_mpidr);
        }
        assert_eq!(regs.reg_read(16), fake_mpidr);
        regs.clear();
        assert_eq!(regs.reg_read(16), 0);

        // L2: set
        // L1: cleared
        // L0: set
        // WEN: cleared
        // PC: cleared
        // PP: cleared
        // WK: Wake by PPONR -> 0b10
        let random_sysr_values = 0b1010_0010 << 24;
        regs.reg_write(16, random_sysr_values);
        {
            let mut fvp_power_controller = regs.fvp_power_controller_for_test();
            
            // Write the fake_mpidr | random_sysr_values because just writing fake_mpidr will overwrite the rest of the
            // sys status Register values. This is a workaround as we are faking the registers.
            let sys_status = fvp_power_controller.get_sys_status(random_sysr_values | fake_mpidr);
            assert!(sys_status.contains(SysStatusRegister::L2));
            assert!(!sys_status.contains(SysStatusRegister::L1));
            assert!(sys_status.contains(SysStatusRegister::L0));
            assert!(!sys_status.contains(SysStatusRegister::WEN));
            assert!(!sys_status.contains(SysStatusRegister::PC));
            assert!(!sys_status.contains(SysStatusRegister::PP));
            assert!(sys_status.contains(SysStatusRegister::WK_COLD_PWR_ON));
        }
    }

    #[test]
    fn pwkupr() {
        let mut regs = FakeFvpPowerControllerRegisters::new();
        let fake_mpidr = 865032;
        let wen_flag = 1 << 31;
        {
            let mut fvp_power_controller = regs.fvp_power_controller_for_test();
            fvp_power_controller.configure_wakeup_requests(fake_mpidr, true);

            assert_eq!(regs.reg_read(0x0C), fake_mpidr | wen_flag);
        }

        {
            let mut fvp_power_controller = regs.fvp_power_controller_for_test();
            fvp_power_controller.configure_wakeup_requests(fake_mpidr, false);

            assert_eq!(regs.reg_read(0x0C), fake_mpidr);
        }
    }

    #[test]
    fn pponr() {
        let mut regs = FakeFvpPowerControllerRegisters::new();
        let fake_mpidr = 865032;
        let mut fvp_power_controller = regs.fvp_power_controller_for_test();
        unsafe {
            fvp_power_controller.power_off_proc(fake_mpidr);
        }
        assert_eq!(regs.reg_read(0x00), fake_mpidr);
    }

    #[test]
    fn ppoffr() {
        let mut regs = FakeFvpPowerControllerRegisters::new();
        let fake_mpidr = 865032;
        let mut fvp_power_controller = regs.fvp_power_controller_for_test();
        unsafe {
            fvp_power_controller.power_on_proc(fake_mpidr);
        }
        assert_eq!(regs.reg_read(0x04), fake_mpidr);
    }

    #[test]
    fn pcoffr() {
        let mut regs = FakeFvpPowerControllerRegisters::new();
        let fake_mpidr = 0b1010_0010_1010_0010_1010_0010;
        let mut fvp_power_controller = regs.fvp_power_controller_for_test();
        unsafe {
            fvp_power_controller.power_off_cluster(fake_mpidr);
        }
        assert_eq!(regs.reg_read(0x08), fake_mpidr);
    }
}
