use crate::{
    descriptors::Desc,
    errors::*,
    objects::{JClass, JMethodID, JStaticMethodID},
    strings::JNIString,
    JNIEnv,
};

unsafe impl<'a, 'c, T, U, V> Desc<'a, JMethodID> for (T, U, V)
where
    T: Desc<'a, JClass<'c>>,
    U: Into<JNIString>,
    V: Into<JNIString>,
{
    type Output = JMethodID;

    fn lookup(self, env: &mut JNIEnv<'a>) -> Result<Self::Output> {
        env.get_method_id(self.0, self.1, self.2)
    }
}

unsafe impl<'a, 'c, T, Signature> Desc<'a, JMethodID> for (T, Signature)
where
    T: Desc<'a, JClass<'c>>,
    Signature: Into<JNIString>,
{
    type Output = JMethodID;

    fn lookup(self, env: &mut JNIEnv<'a>) -> Result<Self::Output> {
        Desc::<JMethodID>::lookup((self.0, "<init>", self.1), env)
    }
}

unsafe impl<'a, 'c, T, U, V> Desc<'a, JStaticMethodID> for (T, U, V)
where
    T: Desc<'a, JClass<'c>>,
    U: Into<JNIString>,
    V: Into<JNIString>,
{
    type Output = JStaticMethodID;

    fn lookup(self, env: &mut JNIEnv<'a>) -> Result<Self::Output> {
        env.get_static_method_id(self.0, self.1, self.2)
    }
}
