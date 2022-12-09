use crate::{
    descriptors::Desc,
    errors::*,
    objects::{AutoLocal, GlobalRef, JClass, JObject},
    strings::JNIString,
    JNIEnv,
};

unsafe impl<'a, T> Desc<'a, JClass<'a>> for T
where
    T: Into<JNIString>,
{
    type Output = AutoLocal<'a, JClass<'a>>;

    fn lookup(self, env: &mut JNIEnv<'a>) -> Result<Self::Output> {
        let class = env.find_class(self)?;
        Ok(env.auto_local(class))
    }
}

unsafe impl<'a, 'o> Desc<'a, JClass<'a>> for JObject<'o> {
    type Output = AutoLocal<'a, JClass<'a>>;

    fn lookup(self, env: &mut JNIEnv<'a>) -> Result<Self::Output> {
        Desc::<JClass>::lookup(&self, env)
    }
}

unsafe impl<'a, 'o, 'ob> Desc<'a, JClass<'a>> for &'ob JObject<'o> {
    type Output = AutoLocal<'a, JClass<'a>>;

    fn lookup(self, env: &mut JNIEnv<'a>) -> Result<Self::Output> {
        Ok(env.auto_local(env.get_object_class(self)?))
    }
}

/// This conversion assumes that the `GlobalRef` is a pointer to a class object.

// TODO: Generify `GlobalRef` and get rid of this `impl`. The transmute is
// sound-ish at the moment (`JClass` is currently `repr(transparent)`
// around `JObject`), but that may change in the future. Moreover, this
// doesn't check if the global reference actually refers to a
// `java.lang.Class` object.
unsafe impl<'a, 'b> Desc<'a, JClass<'static>> for &'b GlobalRef {
    type Output = &'b JClass<'static>;

    fn lookup(self, _: &mut JNIEnv<'a>) -> Result<Self::Output> {
        let obj: &JObject<'static> = self.as_ref();
        Ok(unsafe { std::mem::transmute(obj) })
    }
}
