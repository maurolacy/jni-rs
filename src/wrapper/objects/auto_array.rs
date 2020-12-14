use crate::objects::release_mode::ReleaseMode;
use crate::{errors::*, objects::JObject, sys, JNIEnv};
use jni_sys::jboolean;
use std::ptr::NonNull;

/// Auto-release wrapper for pointer-based generic arrays.
///
/// This wrapper is used to wrap pointers returned by Get<Type>ArrayElements.
///
/// These arrays need to be released through a call to Release<Type>ArrayElements.
/// This wrapper provides automatic array release when it goes out of scope.
pub(crate) struct AutoArray<'a, 'b, T: 'static> {
    /// pub to be accessed from specifically typed subclass
    pub obj: JObject<'a>,
    /// pub to be accessed from specifically typed subclass
    pub ptr: NonNull<T>,
    /// pub to be accessed from specifically typed subclass
    pub mode: ReleaseMode,
    is_copy: bool,
    /// pub to be accessed from specifically typed subclass
    pub env: &'a JNIEnv<'b>,
}

impl<'a, 'b, T: 'static> AutoArray<'a, 'b, T> {
    /// Generic / low level object creation
    pub fn new(
        env: &'a JNIEnv<'b>,
        obj: JObject<'a>,
        ptr: *mut T,
        is_copy: jboolean,
        mode: ReleaseMode,
    ) -> Result<Self> {
        Ok(AutoArray {
            obj,
            ptr: NonNull::new(ptr).ok_or(Error::NullPtr("Non-null ptr expected"))?,
            mode,
            is_copy: is_copy == sys::JNI_TRUE,
            env,
        })
    }

    /// Get a reference to the wrapped pointer
    pub fn as_ptr(&self) -> *mut T {
        self.ptr.as_ptr()
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
}

impl<'a, T> From<&'a AutoArray<'a, '_, T>> for *mut T {
    fn from(other: &'a AutoArray<T>) -> *mut T {
        other.as_ptr()
    }
}

/// Trait to define AutoArray type-dependent parts
/// Implement this in specific Auto<Type>Array, to call the proper Get/Release<Type>ArrayElements JNI methods
pub trait TypeArray<'c, 'd> {
    /// Implement this to call the proper Get<Type>ArrayElements JNI method
    fn new(env: &'c JNIEnv<'d>, obj: JObject<'c>, mode: ReleaseMode) -> Result<Self>
    where
        Self: std::marker::Sized;

    /// Implement this to call the proper Release<Type>ArrayElements JNI method
    fn release(&mut self, mode: i32) -> Result<()>;

    /// Commits the changes to the array, if it is a copy
    fn commit(&mut self) -> Result<()> {
        self.release(sys::JNI_COMMIT)
    }
}
