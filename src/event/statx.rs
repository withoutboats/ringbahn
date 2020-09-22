use std::ffi::CString;
use std::mem::{self, ManuallyDrop};
use std::os::unix::io::RawFd;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use iou::sqe::{StatxFlags, StatxMode};

use super::{Event, SQE, SQEs, Cancellation};

pub struct Statx {
    pub dir_fd: RawFd,
    pub path: CString,
    pub flags: StatxFlags,
    pub mask: StatxMode,
    pub statx: Box<libc::statx>,
}

impl Statx {
    pub fn without_dir(path: impl AsRef<Path>, flags: StatxFlags, mask: StatxMode) -> Statx {
        let path = CString::new(path.as_ref().as_os_str().as_bytes()).unwrap();
        let statx = unsafe { Box::new(mem::zeroed()) };
        Statx { path, dir_fd: libc::AT_FDCWD, flags, mask, statx }
    }
}

impl Event for Statx {
    fn sqes_needed(&self) -> u32 { 1 }

    unsafe fn prepare<'sq>(&mut self, sqs: &mut SQEs<'sq>) -> SQE<'sq> {
        let mut sqe = sqs.single().unwrap();
        sqe.prep_statx(self.dir_fd, self.path.as_c_str(), self.flags, self.mask, &mut *self.statx);
        sqe
    }

    unsafe fn cancel(this: &mut ManuallyDrop<Self>) -> Cancellation {
        unsafe fn callback(addr: *mut (), path: usize) {
            drop(Box::from_raw(addr as *mut libc::statx));
            drop(CString::from_raw(path as *mut libc::c_char))
        }
        Cancellation::new(
            &mut *this.statx as *mut libc::statx as *mut (),
            this.path.as_ptr() as usize,
            callback,
        )
    }
}