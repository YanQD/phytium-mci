mod err;
pub mod constants;
mod regs;

use core::ptr::NonNull;
use constants::*;
use err::*;
use regs::{XReg0,XReg1};
use crate::regs::{FlagReg, Reg};

type IoPadReg = Reg<FioPadError>;

struct IoPad {
    reg: IoPadReg,
    is_ready: bool,
}

impl IoPad {
    pub fn new() -> Self {
        IoPad {
            reg: IoPadReg::new(
                {
                    let addr = PAD_ADDRESS as *mut u8;
                    NonNull::new(addr).unwrap()
                }
            ),
            is_ready: true,
        }
    }

    pub fn func_get<T: FlagReg + XReg0>(&self) -> FioPadFunc {
        let reg_val = self.reg.read_reg::<T>();
        let func = T::func_get(reg_val.into());
        return FioPadFunc::from(func);
    }

    pub fn func_set<T: FlagReg + XReg0>(&mut self, func: FioPadFunc) {
        self.reg.modify_reg::<T>(|reg| {
            (reg.into() | T::func_set(func.into()).into()).into()
        });
    }

    pub fn pull_get<T: FlagReg + XReg0>(&self) -> FioPadPull {
        let reg_val = self.reg.read_reg::<T>();
        let pull = T::pull_get(reg_val.into());
        return FioPadPull::from(pull);
    }

    pub fn pull_set<T: FlagReg + XReg0>(&mut self, pull: FioPadPull) {
        self.reg.modify_reg::<T>(|reg| {
            (reg.into() | T::pull_set(pull.into()).into()).into()
        });
    }

    pub fn drive_get<T: FlagReg + XReg0>(&self) -> FioPadDrive {
        let reg_val = self.reg.read_reg::<T>();
        let drive = T::drive_get(reg_val.into());
        return FioPadDrive::from(drive);
    }

    pub fn drive_set<T: FlagReg + XReg0>(&mut self, drive: FioPadDrive) {
        self.reg.modify_reg::<T>(|reg| {
            (reg.into() | T::drive_set(drive.into()).into()).into()
        });
    }

    pub fn config_set<T: FlagReg + XReg0>(&mut self, func: FioPadFunc, pull: FioPadPull, drive: FioPadDrive) {
        self.reg.modify_reg::<T>(|reg| {
            (T::func_set(func.into()).into() | T::pull_set(pull.into()).into() | T::drive_set(drive.into()).into()).into()
        });
    }

    pub fn config_get <T: FlagReg + XReg0>(&self) -> (FioPadFunc, FioPadPull, FioPadDrive) {
        let reg_val = self.reg.read_reg::<T>();
        let func = T::func_get(reg_val.into());
        let pull = T::pull_get(reg_val.into());
        let drive = T::drive_get(reg_val.into());
        return (FioPadFunc::from(func), FioPadPull::from(pull), FioPadDrive::from(drive));
    }

    pub fn delay_get <T: FlagReg + XReg1>(&self,dir:FioPadDelayDir,typ:FioPadDelayType) -> FioPadDelay {
        let reg_val = self.reg.read_reg::<T>();
        let delay:T;
        if dir == FioPadDelayDir::OutputDelay {
            if typ == FioPadDelayType::DelayFineTuning {
                delay = T::out_delay_delicate_get(reg_val.into());
            } else if typ == FioPadDelayType::DelayCoarseTuning {
                delay = T::out_delay_rough_get(reg_val.into());
            }
        } else if dir == FioPadDelayDir::InputDelay {
            if typ == FioPadDelayType::DelayFineTuning {
                delay = T::in_delay_delicate_get(reg_val.into());
            } else if typ == FioPadDelayType::DelayCoarseTuning {
                delay = T::in_delay_rough_get(reg_val.into());
            }
        }
        delay.into()
    }

    pub fn delay_set <T: FlagReg + XReg1>(&mut self, dir:FioPadDelayDir, typ:FioPadDelayType, delay: FioPadDelay) {
        let reg_val = self.reg.read_reg::<T>();
        if dir == FioPadDelayDir::OutputDelay {
            if typ == FioPadDelayType::DelayFineTuning {
                self.reg.modify_reg::<T>(|reg| {
                    (reg.into() | T::out_delay_delicate_set(delay.into()).into()).into()
                });
            } else if typ == FioPadDelayType::DelayCoarseTuning {
                self.reg.modify_reg::<T>(|reg| {
                    (reg.into() | T::out_delay_rough_set(delay.into()).into()).into()
                });
            }
        } else if dir == FioPadDelayDir::InputDelay {
            if typ == FioPadDelayType::DelayFineTuning {
                self.reg.modify_reg::<T>(|reg| {
                    (reg.into() | T::in_delay_delicate_set(delay.into()).into()).into()
                });
            } else if typ == FioPadDelayType::DelayCoarseTuning {
                self.reg.modify_reg::<T>(|reg| {
                    (reg.into() | T::in_delay_rough_set(delay.into()).into()).into()
                });
            }
        }
    }

}


