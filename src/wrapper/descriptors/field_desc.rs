use crate::{
    descriptors::Desc,
    errors::*,
    objects::{JClass, JFieldID, JStaticFieldID},
    strings::JNIString,
    JNIEnv,
};

unsafe impl<'a, 'c, T, U, V> Desc<'a, JFieldID> for (T, U, V)
where
    T: Desc<'a, JClass<'c>>,
    U: Into<JNIString>,
    V: Into<JNIString>,
{
    type Output = JFieldID;

    fn lookup(self, env: &mut JNIEnv<'a>) -> Result<Self::Output> {
        env.get_field_id(self.0, self.1, self.2)
    }
}

unsafe impl<'a, 'c, T, U, V> Desc<'a, JStaticFieldID> for (T, U, V)
where
    T: Desc<'a, JClass<'c>>,
    U: Into<JNIString>,
    V: Into<JNIString>,
{
    type Output = JStaticFieldID;

    fn lookup(self, env: &mut JNIEnv<'a>) -> Result<Self::Output> {
        env.get_static_field_id(self.0, self.1, self.2)
    }
}
