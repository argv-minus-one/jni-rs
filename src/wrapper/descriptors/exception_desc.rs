use crate::{
    descriptors::Desc,
    errors::*,
    objects::{AutoLocal, JClass, JObject, JThrowable, JValueRef},
    strings::JNIString,
    JNIEnv,
};

const DEFAULT_EXCEPTION_CLASS: &str = "java/lang/RuntimeException";

unsafe impl<'a, 'c, C, M> Desc<'a, JThrowable<'a>> for (C, M)
where
    C: Desc<'a, JClass<'c>>,
    M: Into<JNIString>,
{
    type Output = AutoLocal<'a, JThrowable<'a>>;

    fn lookup(self, env: &mut JNIEnv<'a>) -> Result<Self::Output> {
        let jmsg: AutoLocal<JObject> = env.auto_local(env.new_string(self.1)?.into());
        let obj: JThrowable = env
            .new_object(self.0, "(Ljava/lang/String;)V", &[JValueRef::from(&jmsg)])?
            .into();
        Ok(env.auto_local(obj))
    }
}

unsafe impl<'a> Desc<'a, JThrowable<'a>> for Exception {
    type Output = AutoLocal<'a, JThrowable<'a>>;

    fn lookup(self, env: &mut JNIEnv<'a>) -> Result<Self::Output> {
        Desc::<JThrowable>::lookup((self.class, self.msg), env)
    }
}

unsafe impl<'a, 'b> Desc<'a, JThrowable<'a>> for &'b str {
    type Output = AutoLocal<'a, JThrowable<'a>>;

    fn lookup(self, env: &mut JNIEnv<'a>) -> Result<Self::Output> {
        Desc::<JThrowable>::lookup((DEFAULT_EXCEPTION_CLASS, self), env)
    }
}

unsafe impl<'a> Desc<'a, JThrowable<'a>> for String {
    type Output = AutoLocal<'a, JThrowable<'a>>;

    fn lookup(self, env: &mut JNIEnv<'a>) -> Result<Self::Output> {
        Desc::<JThrowable>::lookup((DEFAULT_EXCEPTION_CLASS, self), env)
    }
}

unsafe impl<'a> Desc<'a, JThrowable<'a>> for JNIString {
    type Output = AutoLocal<'a, JThrowable<'a>>;

    fn lookup(self, env: &mut JNIEnv<'a>) -> Result<Self::Output> {
        Desc::<JThrowable>::lookup((DEFAULT_EXCEPTION_CLASS, self), env)
    }
}
