use crate::{
    errors::*,
    objects::{AutoLocal, JMethodID, JObject, JValue},
    signature::{Primitive, ReturnType},
    sys::jint,
    JNIEnv,
};

/// Wrapper for JObjects that implement `java/util/List`. Provides methods to get,
/// add, and remove elements.
///
/// Looks up the class and method ids on creation rather than for every method
/// call.
pub struct JList<'local: 'obj_ref, 'obj_ref> {
    internal: &'obj_ref JObject<'local>,
    get: JMethodID,
    add: JMethodID,
    add_idx: JMethodID,
    remove: JMethodID,
    size: JMethodID,
}

impl<'local: 'obj_ref, 'obj_ref> AsRef<JList<'local, 'obj_ref>> for JList<'local, 'obj_ref> {
    fn as_ref(&self) -> &JList<'local, 'obj_ref> {
        self
    }
}

impl<'local: 'obj_ref, 'obj_ref> AsRef<JObject<'local>> for JList<'local, 'obj_ref> {
    fn as_ref(&self) -> &JObject<'local> {
        self.internal
    }
}

impl<'local: 'obj_ref, 'obj_ref> JList<'local, 'obj_ref> {
    /// Create a map from the environment and an object. This looks up the
    /// necessary class and method ids to call all of the methods on it so that
    /// exra work doesn't need to be done on every method call.
    pub fn from_env(env: &mut JNIEnv, obj: &'obj_ref JObject<'local>) -> Result<JList<'local, 'obj_ref>> {
        let class = AutoLocal::new(env.find_class("java/util/List")?, env);

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
    pub fn get<'other_local>(&self, env: &mut JNIEnv<'other_local>, idx: jint) -> Result<Option<JObject<'other_local>>> {
        // SAFETY: We keep the class loaded, and fetched the method ID for this function.
        // Provided argument is statically known as a JObject/null, rather than another primitive type.
        let result = unsafe {
            env.call_method_unchecked(
                self.internal,
                self.get,
                ReturnType::Object,
                &[JValue::from(idx).as_jni()],
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
                &[JValue::from(value).as_jni()],
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
                &[JValue::from(idx).as_jni(), JValue::from(value).as_jni()],
            )
        };

        let _ = result?;
        Ok(())
    }

    /// Remove an element from the list by index
    pub fn remove<'other_local>(&self, env: &mut JNIEnv<'other_local>, idx: jint) -> Result<Option<JObject<'other_local>>> {
        // SAFETY: We keep the class loaded, and fetched the method ID for this function.
        // Provided argument is statically known as a int, rather than any other java type.
        let result = unsafe {
            env.call_method_unchecked(
                self.internal,
                self.remove,
                ReturnType::Object,
                &[JValue::from(idx).as_jni()],
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
    pub fn pop<'other_local>(&self, env: &mut JNIEnv<'other_local>) -> Result<Option<JObject<'other_local>>> {
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
                &[JValue::from(size - 1).as_jni()],
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
    pub fn iter<'list>(&'list self, env: &mut JNIEnv) -> Result<JListIter<'list, 'local, 'obj_ref>> {
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
pub struct JListIter<'list, 'local: 'obj_ref, 'obj_ref> {
    list: &'list JList<'local, 'obj_ref>,
    current: jint,
    size: jint,
}

impl<'list, 'local: 'obj_ref, 'obj_ref> JListIter<'list, 'local, 'obj_ref> {
    /// Advances the iterator and returns the next object in the
    /// `java.util.List`, or `None` if there are no more objects.
    ///
    /// This returns:
    ///
    /// * `Ok(Some(_))`: if there was another object in the list.
    /// * `Ok(None)`: if there are no more objects in the list.
    /// * `Err(_)`: if there was an error calling the Java method to
    ///   get the next object.
    ///
    /// This is like [`Iterator::next`], but requires a parameter of
    /// type `&mut JNIEnv` in order to call into Java.
    pub fn next<'other_local>(&mut self, env: &mut JNIEnv<'other_local>) -> Result<Option<JObject<'other_local>>> {
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
