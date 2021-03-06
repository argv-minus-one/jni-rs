use crate::sys::{jbyte, jsize};
use log::error;

use crate::objects::release_mode::ReleaseMode;
use crate::{errors::*, objects::JObject, sys, JNIEnv};
use std::ptr::NonNull;

/// Auto-release wrapper for pointer-based byte arrays.
///
/// This wrapper is used to wrap pointers returned by get_byte_array_elements.
///
/// These arrays need to be released through a call to release_byte_array_elements.
/// This wrapper provides automatic array release when it goes out of scope.
pub struct AutoByteArray<'a: 'b, 'b> {
    obj: JObject<'a>,
    ptr: NonNull<jbyte>,
    mode: ReleaseMode,
    is_copy: bool,
    env: &'b JNIEnv<'a>,
}

impl<'a, 'b> AutoByteArray<'a, 'b> {
    /// Creates a new auto-release wrapper for a pointer-based byte array
    ///
    /// Once this wrapper goes out of scope, `release_byte_array_elements` will be
    /// called on the object. While wrapped, the object can be accessed via
    /// the `From` impl.
    pub(crate) fn new(
        env: &'b JNIEnv<'a>,
        obj: JObject<'a>,
        ptr: *mut jbyte,
        mode: ReleaseMode,
        is_copy: bool,
    ) -> Result<Self> {
        Ok(AutoByteArray {
            obj,
            ptr: NonNull::new(ptr).ok_or(Error::NullPtr("Non-null ptr expected"))?,
            mode,
            is_copy,
            env,
        })
    }

    /// Get a reference to the wrapped pointer
    pub fn as_ptr(&self) -> *mut jbyte {
        self.ptr.as_ptr()
    }

    /// Commits the changes to the array, if it is a copy
    pub fn commit(&mut self) -> Result<()> {
        self.release_byte_array_elements(sys::JNI_COMMIT)
    }

    fn release_byte_array_elements(&mut self, mode: i32) -> Result<()> {
        jni_void_call!(
            self.env.get_native_interface(),
            ReleaseByteArrayElements,
            *self.obj,
            self.ptr.as_mut(),
            mode
        );
        Ok(())
    }

    /// Don't commit the changes to the array on release (if it is a copy).
    /// This has no effect if the array is not a copy.
    /// This method is useful to change the release mode of an array originally created
    /// with `ReleaseMode::CopyBack`.
    pub fn discard(&mut self) {
        self.mode = ReleaseMode::NoCopyBack;
    }

    /// Indicates if the array is a copy or not
    pub fn is_copy(&self) -> bool {
        self.is_copy
    }

    /// Returns the array size
    pub fn size(&self) -> Result<jsize> {
        self.env.get_array_length(*self.obj)
    }
}

impl<'a, 'b> Drop for AutoByteArray<'a, 'b> {
    fn drop(&mut self) {
        let res = self.release_byte_array_elements(self.mode as i32);
        match res {
            Ok(()) => {}
            Err(e) => error!("error releasing byte array: {:#?}", e),
        }
    }
}

impl<'a> From<&'a AutoByteArray<'a, '_>> for *mut jbyte {
    fn from(other: &'a AutoByteArray) -> *mut jbyte {
        other.as_ptr()
    }
}
