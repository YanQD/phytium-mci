use core::cell::RefCell;
use alloc::rc::Rc;

use super::MCIHost;

struct SdMmcBase {
    host: Option<Rc<RefCell<MCIHost>>>,
}