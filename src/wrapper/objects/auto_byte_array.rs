use log::debug;

use crate::sys::{jboolean, jbyte};
use crate::wrapper::objects::auto_array::{AutoArray, TypeArray};
use crate::wrapper::objects::ReleaseMode;
use crate::{errors::*, objects::JObject, JNIEnv};
use std::ops::Deref;

/// Auto-release wrapper for pointer-based byte arrays.
pub struct AutoByteArray<'a: 'b, 'b>(pub(crate) AutoArray<'a, 'b, jbyte>);

impl<'a, 'b> AutoByteArray<'a, 'b> {
    /// Commits the changes to the array, if it is a copy
    fn commit(&mut self) -> Result<()> {
        TypeArray::commit(self)
    }
}

impl<'a, 'b> TypeArray<'a, 'b> for AutoByteArray<'a, 'b> {
    /// Creates a new auto-release wrapper for a pointer-based byte array
    ///
    /// Once this wrapper goes out of scope, `ReleaseByteArrayElements` will be
    /// called on the object. While wrapped, the object can be accessed via
    /// the `From` impl.
    fn new(env: &'a JNIEnv<'b>, obj: JObject<'a>, mode: ReleaseMode) -> Result<Self> {
        let mut is_copy: jboolean = 0xff;
        let internal = env.get_native_interface();
        let ptr = jni_non_void_call!(internal, GetByteArrayElements, *obj, &mut is_copy);
        Ok(AutoByteArray(AutoArray::new(env, obj, ptr, is_copy, mode)?))
    }

    fn release(&mut self, mode: i32) -> Result<()> {
        let env = self.0.env.get_native_interface();
        let ptr = self.0.ptr.as_ptr();
        jni_void_call!(
            env,
            ReleaseByteArrayElements,
            *self.0.obj,
            ptr as *mut i8,
            mode
        );
        Ok(())
    }
}

impl<'a, 'b> Deref for AutoByteArray<'a, 'b> {
    type Target = AutoArray<'a, 'b, jbyte>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, 'b> Drop for AutoByteArray<'a, 'b> {
    fn drop(&mut self) {
        let res = self.release(self.0.mode as i32);
        match res {
            Ok(()) => {}
            Err(e) => debug!("error releasing array: {:#?}", e),
        }
    }
}

impl<'a, 'b> From<&AutoByteArray<'a, 'b>> for *mut jbyte {
    fn from(other: &AutoByteArray<'a, 'b>) -> *mut jbyte {
        other.as_ptr()
    }
}