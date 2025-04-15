use core::{alloc::Layout, ptr::NonNull};

use err::FMempStatus;
use log::{error, warn};
use tlsf::*;
use spin::Mutex;
use lazy_static::*;

mod err;

static mut MEMORY: [u8; 4096] = [0; 4096];

pub struct FMemp {
    tlsf_ptr: Option<Tlsf>,
    is_ready: bool,
}

lazy_static! {
    pub static ref GLOBAL_FMEMP: Mutex<FMemp> = 
        Mutex::new(FMemp::new());
}

// pub static mut GLOBAL_FMEMP: FMemp = {
//     FMemp {
//         tlsf_ptr: None,
//         is_ready: false,
//     }
// };

impl FMemp {
    pub fn new() -> Self {
        Self {
            tlsf_ptr: None,
            is_ready: false,
        }
    }

    pub fn tlsf_ptr_mut(&mut self) -> Option<&mut Tlsf> {
        self.tlsf_ptr.as_mut()
    }

    pub fn is_ready(&self) -> bool {
        self.is_ready
    }

    pub fn fmemp_init(&mut self) -> FMempStatus {
        if self.is_ready() {
            warn!("Memory pool already inited!");
            return Ok(());
        }

        if self.tlsf_ptr.is_none() {
            let mut tlsf = Tlsf::new();

            unsafe {
                if tlsf.extend(&mut MEMORY) == 0 {
                    error!("Allocate TLSF buf failed!");
                    return Err(err::FMempError::InitTlsfError);
                }
                self.tlsf_ptr = Some(tlsf);
            }
        } else {
            warn!("TLSF buf aleady exists, skip from allocate!")
        }

        self.is_ready = true;

        Ok(())
    }

    pub fn osa_malloc_align(&mut self, length: u32, align: usize) -> Option<NonNull<u8>> {
        let size = if length > 20 { length } else { 20 };
        if self.tlsf_ptr.is_some() {
            let layout = Layout::from_size_align(size as usize, align).unwrap();
            let ret = unsafe { self.tlsf_ptr_mut().unwrap().alloc(layout) };
            if ret.is_err() {
                error!("osa malloc aligned failed!");
                return None;
            } else {
                return Some(ret.unwrap());
            }
        }
        None
    }
}

pub fn osa_init() {
    if GLOBAL_FMEMP.lock().fmemp_init().is_err() {
        panic!("OSA init failed!");
    }
}

pub fn osa_deinit() {

}

pub fn osa_alloc(length: u32) -> Option<NonNull<u8>> {
    GLOBAL_FMEMP.lock().osa_malloc_align(length, 8)
}

// #[bare_test::tests]
// mod tests {
//     use super::*;

//     // 模拟裸机环境下的内存区域
//     #[test]
//     fn test_fmemp_init() {
//         unsafe {
//             let result = GLOBAL_FMEMP.fmemp_init();

//             // 检查初始化是否成功
//             assert!(result.is_ok());
//             assert!(GLOBAL_FMEMP.is_ready);
//         }
//     }
// }