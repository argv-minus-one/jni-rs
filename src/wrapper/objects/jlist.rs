use crate::{
    errors::*,
    objects::{JMethodID, JObject, JValue},
    signature::{Primitive, ReturnType},
    sys::jint,
    JNIEnv,
};

/// Wrapper for JObjects that implement `java/util/List`. Provides methods to get,
/// add, and remove elements.
///
/// Looks up the class and method ids on creation rather than for every method
/// call.
pub struct JList<'a: 'b, 'b> {
    internal: &'b JObject<'a>,
    get: JMethodID,
    add: JMethodID,
    add_idx: JMethodID,
    remove: JMethodID,
    size: JMethodID,
}

impl<'a: 'b, 'b> AsRef<JList<'a, 'b>> for JList<'a, 'b> {
    fn as_ref(&self) -> &JList<'a, 'b> {
        self
    }
}

impl<'a: 'b, 'b> AsRef<JObject<'a>> for JList<'a, 'b> {
    fn as_ref(&self) -> &JObject<'a> {
        self.internal
    }
}

impl<'a: 'b, 'b> JList<'a, 'b> {
    /// Create a map from the environment and an object. This looks up the
    /// necessary class and method ids to call all of the methods on it so that
    /// exra work doesn't need to be done on every method call.
    pub fn from_env(env: &mut JNIEnv, obj: &'b JObject<'a>) -> Result<JList<'a, 'b>> {
        let class = env.auto_local(env.find_class("java/util/List")?);

        let get = env.get_method_id(&class, "get", "(I)Ljava/lang/Object;")?;
        let add = env.get_method_id(&class, "add", "(Ljava/lang/Object;)Z")?;
        let add_idx = env.get_method_id(&class, "add", "(ILjava/lang/Object;)V")?;
        let remove = env.get_method_id(&class, "remove", "(I)Ljava/lang/Object;")?;
        let size = env.get_method_id(&class, "size", "()I")?;

        Ok(JList {
            internal: obj,
            get,
            add,
            add_idx,
            remove,
            size,
        })
    }

    /// Look up the value for a key. Returns `Some` if it's found and `None` if
    /// a null pointer would be returned.
    pub fn get<'a2>(&self, env: &mut JNIEnv<'a2>, idx: jint) -> Result<Option<JObject<'a2>>> {
        // SAFETY: We keep the class loaded, and fetched the method ID for this function.
        // Provided argument is statically known as a JObject/null, rather than another primitive type.
        let result = unsafe {
            env.call_method_unchecked(
                self.internal,
                self.get,
                ReturnType::Object,
                &[JValue::from(idx).to_jni()],
            )
        };

        match result {
            Ok(val) => Ok(Some(val.l()?)),
            Err(e) => match e {
                Error::NullPtr(_) => Ok(None),
                _ => Err(e),
            },
        }
    }

    /// Append an element to the list
    pub fn add(&self, env: &mut JNIEnv, value: &JObject) -> Result<()> {
        // SAFETY: We keep the class loaded, and fetched the method ID for this function.
        // Provided argument is statically known as a JObject/null, rather than another primitive type.
        let result = unsafe {
            env.call_method_unchecked(
                self.internal,
                self.add,
                ReturnType::Primitive(Primitive::Boolean),
                &[JValue::from(value).to_jni()],
            )
        };

        let _ = result?;
        Ok(())
    }

    /// Insert an element at a specific index
    pub fn insert(&self, env: &mut JNIEnv, idx: jint, value: &JObject) -> Result<()> {
        // SAFETY: We keep the class loaded, and fetched the method ID for this function.
        // Provided argument is statically known as a JObject/null, rather than another primitive type.
        let result = unsafe {
            env.call_method_unchecked(
                self.internal,
                self.add_idx,
                ReturnType::Primitive(Primitive::Void),
                &[JValue::from(idx).to_jni(), JValue::from(value).to_jni()],
            )
        };

        let _ = result?;
        Ok(())
    }

    /// Remove an element from the list by index
    pub fn remove<'a2>(&self, env: &mut JNIEnv<'a2>, idx: jint) -> Result<Option<JObject<'a2>>> {
        // SAFETY: We keep the class loaded, and fetched the method ID for this function.
        // Provided argument is statically known as a int, rather than any other java type.
        let result = unsafe {
            env.call_method_unchecked(
                self.internal,
                self.remove,
                ReturnType::Object,
                &[JValue::from(idx).to_jni()],
            )
        };

        match result {
            Ok(val) => Ok(Some(val.l()?)),
            Err(e) => match e {
                Error::NullPtr(_) => Ok(None),
                _ => Err(e),
            },
        }
    }

    /// Get the size of the list
    pub fn size(&self, env: &mut JNIEnv) -> Result<jint> {
        // SAFETY: We keep the class loaded, and fetched the method ID for this function.
        let result = unsafe {
            env.call_method_unchecked(
                self.internal,
                self.size,
                ReturnType::Primitive(Primitive::Int),
                &[],
            )
        };

        result.and_then(|v| v.i())
    }

    /// Pop the last element from the list
    ///
    /// Note that this calls `size()` to determine the last index.
    pub fn pop<'a2>(&self, env: &mut JNIEnv<'a2>) -> Result<Option<JObject<'a2>>> {
        let size = self.size(env)?;
        if size == 0 {
            return Ok(None);
        }

        // SAFETY: We keep the class loaded, and fetched the method ID for this function.
        // Provided argument is statically known as a int.
        let result = unsafe {
            env.call_method_unchecked(
                self.internal,
                self.remove,
                ReturnType::Object,
                &[JValue::from(size - 1).to_jni()],
            )
        };

        match result {
            Ok(val) => Ok(Some(val.l()?)),
            Err(e) => match e {
                Error::NullPtr(_) => Ok(None),
                _ => Err(e),
            },
        }
    }

    /// Get key/value iterator for the map. This is done by getting the
    /// `EntrySet` from java and iterating over it.
    pub fn iter<'list>(&'list self, env: &mut JNIEnv) -> Result<JListIter<'list, 'a, 'b>> {
        Ok(JListIter {
            list: self,
            current: 0,
            size: self.size(env)?,
        })
    }
}

/// An iterator over the keys and values in a map.
///
/// TODO: make the iterator implementation for java iterators its own thing
/// and generic enough to use elsewhere.
pub struct JListIter<'list, 'a: 'b, 'b> {
    list: &'list JList<'a, 'b>,
    current: jint,
    size: jint,
}

impl<'list, 'a: 'b, 'b> JListIter<'list, 'a, 'b> {
    pub fn next<'a2>(&mut self, env: &mut JNIEnv<'a2>) -> Result<Option<JObject<'a2>>> {
        if self.current == self.size {
            return Ok(None);
        }

        let res = self.list.get(env, self.current);

        self.current = match &res {
            Ok(Some(_)) => self.current + 1,
            Ok(None) => self.current,
            Err(_) => self.size,
        };

        res
    }
}
