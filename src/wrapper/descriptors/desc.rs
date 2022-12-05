use crate::{errors::*, JNIEnv, objects::{AutoLocal, JObject}};

#[cfg(doc)]
use crate::objects::{JClass, JMethodID};

/// Trait for things that can be looked up through the JNI via a descriptor.
/// This will be something like the fully-qualified class name
/// `java/lang/String` or a tuple containing a class descriptor, method name,
/// and method signature. For convenience, this is also implemented for the
/// concrete types themselves in addition to their descriptors.
///
/// # Safety
///
/// Implementations of this trait must return the correct value from the
/// `lookup` method. It must not, for example, return a random [`JMethodID`] or
/// the [`JClass`] of a class other than the one requested. Returning such an
/// incorrect value results in undefined behavior. This requirement also
/// applies to the returned value's implementation of `AsRef<T>`.
pub unsafe trait Desc<'a, T> {
    /// The type that this `Desc` returns.
    type Output: AsRef<T>;

    /// Look up the concrete type from the JVM.
    fn lookup(self, _: &mut JNIEnv<'a>) -> Result<Self::Output>;
}

unsafe impl<'a, T> Desc<'a, T> for T
where
    T: AsRef<T>,
{
    type Output = Self;

    fn lookup(self, _: &mut JNIEnv<'a>) -> Result<T> {
        Ok(self)
    }
}

unsafe impl<'a, 'b, T> Desc<'a, T> for &'b T
where
    T: AsRef<T>,
{
    type Output = Self;

    fn lookup(self, _: &mut JNIEnv<'a>) -> Result<Self::Output> {
        Ok(self)
    }
}

unsafe impl<'a, 'a2, T> Desc<'a, T> for AutoLocal<'a2, T>
where
    T: AsRef<T> + Into<JObject<'a2>>,
{
    type Output = Self;

    fn lookup(self, _: &mut JNIEnv<'a>) -> Result<Self::Output> {
        Ok(self)
    }
}

unsafe impl<'a, 'a2, 'b, T> Desc<'a, T> for &'b AutoLocal<'a2, T>
where
    T: AsRef<T> + Into<JObject<'a2>>,
{
    type Output = Self;

    fn lookup(self, _: &mut JNIEnv<'a>) -> Result<Self::Output> {
        Ok(self)
    }
}
