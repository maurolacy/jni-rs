use log::debug;

use crate::sys::{jboolean, jlong};
use crate::wrapper::objects::auto_array::{AutoArray, TypeArray};
use crate::wrapper::objects::ReleaseMode;
use crate::{errors::*, objects::JObject, JNIEnv};

/// Auto-release wrapper for pointer-based long arrays.
pub struct AutoLongArray<'a: 'b, 'b>(pub(crate) AutoArray<'a, 'b, jlong>);

impl<'a, 'b> AutoLongArray<'a, 'b> {
    /// Get a reference to the wrapped pointer
    pub fn as_ptr(&self) -> *mut jlong {
        self.0.as_ptr()
    }

    /// Discard the changes to the long array if it is a copy
    pub fn discard(&mut self) {
        self.0.discard();
    }

    /// Indicates if the array is a copy or not
    pub fn is_copy(&self) -> bool {
        self.0.is_copy()
    }

    /// Commits the changes to the array, if it is a copy
    pub fn commit(&mut self) -> Result<()> {
        TypeArray::commit(self)
    }
}

impl<'a, 'b> TypeArray<'a, 'b> for AutoLongArray<'a, 'b> {
    /// Creates a new auto-release wrapper for a pointer-based long array
    ///
    /// Once this wrapper goes out of scope, `ReleaseLongArrayElements` will be
    /// called on the object. While wrapped, the object can be accessed via
    /// the `From` impl.
    fn new(env: &'a JNIEnv<'b>, obj: JObject<'a>, mode: ReleaseMode) -> Result<Self> {
        let mut is_copy: jboolean = 0xff;
        let internal = env.get_native_interface();
        let ptr = jni_non_void_call!(internal, GetLongArrayElements, *obj, &mut is_copy);
        Ok(AutoLongArray(AutoArray::new(env, obj, ptr, is_copy, mode)?))
    }

    fn release(&mut self, mode: i32) -> Result<()> {
        let env = self.0.env.get_native_interface();
        let ptr = self.0.ptr.as_ptr();
        jni_void_call!(
            env,
            ReleaseLongArrayElements,
            *self.0.obj,
            ptr as *mut i64,
            mode
        );
        Ok(())
    }
}

impl<'a, 'b> Drop for AutoLongArray<'a, 'b> {
    fn drop(&mut self) {
        let res = self.release(self.0.mode as i32);
        match res {
            Ok(()) => {}
            Err(e) => debug!("error releasing array: {:#?}", e),
        }
    }
}

impl<'a, 'b> From<&AutoLongArray<'a, 'b>> for *mut jlong {
    fn from(other: &AutoLongArray<'a, 'b>) -> *mut jlong {
        other.as_ptr()
    }
}
