use std::{
    mem,
    ops::{Deref, DerefMut},
    ptr,
};

use log::debug;

use crate::{objects::JObject, JNIEnv};

/// Auto-delete wrapper for local refs.
///
/// Anything passed to a foreign method _and_ returned from JNI methods is considered a local ref
/// unless it is specified otherwise.
/// These refs are automatically deleted once the foreign method exits, but it's possible that
/// they may reach the JVM-imposed limit before that happens.
///
/// This wrapper provides automatic local ref deletion when it goes out of
/// scope.
///
/// See also the [JNI specification][spec-references] for details on referencing Java objects
/// and some [extra information][android-jni-references].
///
/// [spec-references]: https://docs.oracle.com/en/java/javase/12/docs/specs/jni/design.html#referencing-java-objects
/// [android-jni-references]: https://developer.android.com/training/articles/perf-jni#local-and-global-references
#[derive(Debug)]
pub struct AutoLocal<'a, T> {
    obj: T,
    env: JNIEnv<'a>,
}

impl<'a, T> AutoLocal<'a, T> {
    /// Creates a new auto-delete wrapper for a local ref.
    ///
    /// Once this wrapper goes out of scope, the `delete_local_ref` will be
    /// called on the object. While wrapped, the object can be accessed via
    /// the `Deref` impl.
    pub fn new(env: &JNIEnv<'a>, obj: T) -> Self
    where
        T: AsMut<JObject<'a>>,
    {
        Self::new_unchecked(env, obj)
    }

    /// Creates a new auto-delete wrapper for a local ref.
    ///
    /// The resulting [`AutoLocal`] will only call [`JNIEnv::delete_local_ref`] on `obj` if its
    /// type, `T`, implements <code>[AsMut]&lt;[JObject]&lt;'a>></code>. If not, `obj` will be
    /// dropped without calling `delete_local_ref`. This function is useful in generic code that
    /// needs to accept both owned and borrowed `JObject`s, and auto-delete owned `JObject`s.
    ///
    /// This function is not unsafe, but incorrect usage may result in a memory leak. Use
    /// [`AutoLocal::new`] instead, when possible.
    pub fn new_unchecked(env: &JNIEnv<'a>, obj: T) -> Self {
        // Safety: The cloned `JNIEnv` will not be used to create any local references, only to
        // delete one.
        let env = unsafe { env.unsafe_clone() };

        AutoLocal { obj, env }
    }

    /// Forget the wrapper, returning the original object.
    ///
    /// This prevents `delete_local_ref` from being called when the `AutoLocal`
    /// gets
    /// dropped. You must either remember to delete the local ref manually, or
    /// be
    /// ok with it getting deleted once the foreign method returns.
    pub fn forget(mut self) -> T {
        // We need to move `self.obj` out of `self`. Normally that's trivial, but moving out of a
        // type with a `Drop` implementation is not allowed. We'll have to do it manually (and
        // carefully) with `unsafe`.
        //
        // This could be done without `unsafe` by adding `where T: Default` and using
        // `mem::replace` to extract `self.obj`, but doing it this way avoids unnecessarily running
        // the drop routine on `self`.

        let obj = unsafe {
            // Drop the `JNIEnv` in place. As of this writing, that's a no-op, but if `JNIEnv`
            // gains any drop code in the future, this will run it.
            //
            // Safety: The `&mut` proves that `self.env` is valid and not aliased. It is not
            // accessed again after this point. The `mem::forget` below prevents it from being
            // dropped twice.
            ptr::drop_in_place(&mut self.env);

            // Move `obj` out of `self`.
            //
            // Safety: The `&mut` proves that `self.obj` is valid and not aliased. It is not
            // accessed again after this point. The `mem::forget` below prevents it from being
            // dropped after it is moved.
            ptr::read(&mut self.obj)
        };

        // Now that we've done that, `self` being dropped normally would trigger undefined
        // behavior, so we need to prevent that from happening.
        mem::forget(self);

        // Return the extracted `T`.
        obj
    }
}

impl<'a, T> Drop for AutoLocal<'a, T>
where
    T: AsMut<JObject<'a>>,
{
    fn drop(&mut self) {
        let obj: JObject<'a> = mem::take(self.obj.as_mut());

        let res = self.env.delete_local_ref(obj);
        match res {
            Ok(()) => {}
            Err(e) => debug!("error dropping global ref: {:#?}", e),
        }
    }
}

impl<'a, T, U> AsRef<U> for AutoLocal<'a, T>
where
    T: AsRef<U>,
{
    fn as_ref(&self) -> &U {
        self.obj.as_ref()
    }
}

impl<'a, T, U> AsMut<U> for AutoLocal<'a, T>
where
    T: AsMut<U>,
{
    fn as_mut(&mut self) -> &mut U {
        self.obj.as_mut()
    }
}

impl<'a, T> Deref for AutoLocal<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.obj
    }
}

impl<'a, T> DerefMut for AutoLocal<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.obj
    }
}
