use std::{
    ffi::{c_char, CStr, CString},
    marker::PhantomData,
    os::unix::ffi::OsStrExt,
    path::Path,
};

#[repr(C)]
pub struct ZkCutsceneLibrary(
    [u8; 0],
    core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
);
#[repr(C)]
pub struct ZkCutsceneBlock(
    [u8; 0],
    core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
);
#[repr(C)]
struct ZkCutsceneMessage(
    [u8; 0],
    core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
);

extern "C" {
    fn ZkCutsceneLibrary_loadPath(buf: *const c_char) -> *mut ZkCutsceneLibrary;
    fn ZkCutsceneLibrary_del(slf: *mut ZkCutsceneLibrary);
    fn ZkCutsceneLibrary_getBlockCount(slf: *const ZkCutsceneLibrary) -> u64;
    fn ZkCutsceneLibrary_getBlockByIndex(
        slf: *const ZkCutsceneLibrary,
        id: u64,
    ) -> *const ZkCutsceneBlock;

    fn ZkCutsceneBlock_getName(slf: *const ZkCutsceneBlock) -> *const c_char;
    fn ZkCutsceneBlock_getMessage(slf: *const ZkCutsceneBlock) -> *const ZkCutsceneMessage;

    fn ZkCutsceneMessage_getType(slf: *const ZkCutsceneMessage) -> u32;
    fn ZkCutsceneMessage_getText(slf: *const ZkCutsceneMessage) -> *const c_char;
    fn ZkCutsceneMessage_getName(slf: *const ZkCutsceneMessage) -> *const c_char;
}

pub struct CutsceneLibrary(*mut ZkCutsceneLibrary);

impl CutsceneLibrary {
    pub fn load_path(path: impl AsRef<Path>) -> Self {
        let path = CString::new(path.as_ref().as_os_str().as_bytes()).unwrap();

        let csl = unsafe { ZkCutsceneLibrary_loadPath(path.as_ptr()) };

        Self(csl)
    }

    pub fn block_count(&self) -> u64 {
        unsafe { ZkCutsceneLibrary_getBlockCount(self.0) }
    }

    pub fn block_by_index(&self, id: u64) -> Option<CutsceneBlock> {
        let block = unsafe { ZkCutsceneLibrary_getBlockByIndex(self.0, id) };
        if block.is_null() {
            None
        } else {
            Some(CutsceneBlock(block, PhantomData))
        }
    }
}

impl Drop for CutsceneLibrary {
    fn drop(&mut self) {
        unsafe {
            ZkCutsceneLibrary_del(self.0);
        }
    }
}

pub struct CutsceneBlock<'a>(*const ZkCutsceneBlock, PhantomData<&'a ()>);

impl<'a> CutsceneBlock<'a> {
    pub fn name(&self) -> &CStr {
        unsafe {
            let name = ZkCutsceneBlock_getName(self.0);
            CStr::from_ptr(name)
        }
    }

    pub fn message(&self) -> CutsceneMessage<'a> {
        unsafe {
            let msg = ZkCutsceneBlock_getMessage(self.0);
            CutsceneMessage(msg, PhantomData)
        }
    }
}

pub struct CutsceneMessage<'a>(*const ZkCutsceneMessage, PhantomData<&'a ()>);

impl<'a> CutsceneMessage<'a> {
    pub fn ty(&self) -> u32 {
        unsafe { ZkCutsceneMessage_getType(self.0) }
    }

    pub fn text(&self) -> &CStr {
        unsafe {
            let name = ZkCutsceneMessage_getText(self.0);
            CStr::from_ptr(name)
        }
    }

    pub fn name(&self) -> &CStr {
        unsafe {
            let name = ZkCutsceneMessage_getName(self.0);
            CStr::from_ptr(name)
        }
    }
}
