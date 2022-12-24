use crate::{
    objects::JObject,
    sys::{jclass, jobject},
};

/// Lifetime'd representation of a `jclass`. Just a `JObject` wrapped in a new
/// class.
#[repr(transparent)]
#[derive(Debug)]
pub struct JClass<'local>(JObject<'local>);

impl<'local> AsRef<JClass<'local>> for JClass<'local> {
    fn as_ref(&self) -> &JClass<'local> {
        self
    }
}

impl<'local> AsRef<JObject<'local>> for JClass<'local> {
    fn as_ref(&self) -> &JObject<'local> {
        &**self
    }
}

impl<'local> ::std::ops::Deref for JClass<'local> {
    type Target = JObject<'local>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'local> From<JClass<'local>> for JObject<'local> {
    fn from(other: JClass) -> JObject {
        other.0
    }
}

/// This conversion assumes that the `JObject` is a pointer to a class object.
impl<'local> From<JObject<'local>> for JClass<'local> {
    fn from(other: JObject) -> Self {
        unsafe { Self::from_raw(other.into_raw()) }
    }
}

impl<'local> std::default::Default for JClass<'local> {
    fn default() -> Self {
        Self(JObject::null())
    }
}

impl<'local> JClass<'local> {
    /// Creates a [`JClass`] that wraps the given `raw` [`jclass`]
    ///
    /// # Safety
    ///
    /// Expects a valid pointer or `null`
    pub unsafe fn from_raw(raw: jclass) -> Self {
        Self(JObject::from_raw(raw as jobject))
    }

    /// Unwrap to the raw jni type.
    pub fn into_raw(self) -> jclass {
        self.0.into_raw() as jclass
    }
}
