use crate::{
    errors::*,
    objects::{AutoLocal, JClass, JMethodID, JObject, JValue},
    signature::{Primitive, ReturnType},
    JNIEnv,
};

use std::marker::PhantomData;

/// Wrapper for JObjects that implement `java/util/Map`. Provides methods to get
/// and set entries and a way to iterate over key/value pairs.
///
/// Looks up the class and method ids on creation rather than for every method
/// call.
pub struct JMap<'a, O> {
    internal: O,
    class: AutoLocal<'a, JClass<'a>>,
    get: JMethodID,
    put: JMethodID,
    remove: JMethodID,
}

impl<'a, O> AsRef<O> for JMap<'a, O> {
    fn as_ref(&self) -> &O {
        &*self
    }
}

impl<'a, O> ::std::ops::Deref for JMap<'a, O> {
    type Target = O;

    fn deref(&self) -> &Self::Target {
        &self.internal
    }
}

impl<'a, O> From<JMap<'a, O>> for O {
    fn from(other: JMap<'a, O>) -> Self {
        other.internal
    }
}

impl<'a, O> JMap<'a, O>
where
    O: AsRef<JObject<'a>>,
{
    /// Create a map from the environment and an object. This looks up the
    /// necessary class and method ids to call all of the methods on it so that
    /// exra work doesn't need to be done on every method call.
    pub fn from_env(env: &mut JNIEnv, obj: O) -> Result<JMap<'a, O>> {
        let class = env.auto_local(env.find_class("java/util/Map")?);

        let get = env.get_method_id(&class, "get", "(Ljava/lang/Object;)Ljava/lang/Object;")?;
        let put = env.get_method_id(
            &class,
            "put",
            "(Ljava/lang/Object;Ljava/lang/Object;\
             )Ljava/lang/Object;",
        )?;

        let remove =
            env.get_method_id(&class, "remove", "(Ljava/lang/Object;)Ljava/lang/Object;")?;

        Ok(JMap {
            internal: obj,
            class,
            get,
            put,
            remove,
        })
    }

    /// Look up the value for a key. Returns `Some` if it's found and `None` if
    /// a null pointer would be returned.
    pub fn get<'b>(&self, env: &mut JNIEnv<'b>, key: &JObject) -> Result<Option<JObject<'b>>> {
        // SAFETY: We keep the class loaded, and fetched the method ID for this function.
        // Provided argument is statically known as a JObject/null, rather than another primitive type.
        let result = unsafe {
            self.env.call_method_unchecked(
                self.internal,
                self.get,
                ReturnType::Object,
                &[JValue::from(key).to_jni()],
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

    /// Look up the value for a key. Returns `Some` with the old value if the
    /// key already existed and `None` if it's a new key.
    pub fn put<'b>(&self, env: &mut JNIEnv<'b>, key: &JObject, value: &JObject) -> Result<Option<JObject<'b>>> {
        // SAFETY: We keep the class loaded, and fetched the method ID for this function.
        // Provided argument is statically known as a JObject/null, rather than another primitive type.
        let result = unsafe {
            self.env.call_method_unchecked(
                self.internal,
                self.put,
                ReturnType::Object,
                &[JValue::from(key).to_jni(), JValue::from(value).to_jni()],
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

    /// Remove a value from the map. Returns `Some` with the removed value and
    /// `None` if there was no value for the key.
    pub fn remove<'b>(&self, env: &mut JNIEnv<'b>, key: &JObject) -> Result<Option<JObject<'b>>> {
        // SAFETY: We keep the class loaded, and fetched the method ID for this function.
        // Provided argument is statically known as a JObject/null, rather than another primitive type.
        let result = unsafe {
            self.env.call_method_unchecked(
                self.internal,
                self.remove,
                ReturnType::Object,
                &[JValue::from(key).to_jni()],
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
    pub fn iter<'map, 'b>(&'map self, env: &mut JNIEnv<'b>) -> Result<JMapIter<'map, 'a, 'b, O>> {
        let iter_class = self
            .env
            .auto_local(self.env.find_class("java/util/Iterator")?);

        let has_next = self.env.get_method_id(&iter_class, "hasNext", "()Z")?;

        let next = self
            .env
            .get_method_id(&iter_class, "next", "()Ljava/lang/Object;")?;

        let entry_class = self
            .env
            .auto_local(self.env.find_class("java/util/Map$Entry")?);

        let get_key = self
            .env
            .get_method_id(&entry_class, "getKey", "()Ljava/lang/Object;")?;

        let get_value = self
            .env
            .get_method_id(&entry_class, "getValue", "()Ljava/lang/Object;")?;

        // Get the iterator over Map entries.
        // Use the local frame till #109 is resolved, so that implicitly looked-up
        // classes are freed promptly.
        let iter = self.env.with_local_frame(16, || {
            // SAFETY: We keep the class loaded, and fetched the method ID for this function. Arg list is known empty.
            let entry_set = unsafe {
                self.env.call_method_unchecked(
                    self.internal,
                    (&self.class, "entrySet", "()Ljava/util/Set;"),
                    ReturnType::Object,
                    &[],
                )
            }?
            .l()?;

            // SAFETY: We keep the class loaded, and fetched the method ID for this function. Arg list is known empty.
            let iter = unsafe {
                self.env.call_method_unchecked(
                    entry_set,
                    ("java/util/Set", "iterator", "()Ljava/util/Iterator;"),
                    ReturnType::Object,
                    &[],
                )
            }?
            .l()?;

            Ok(iter)
        })?;
        let iter = self.env.auto_local(iter);

        Ok(JMapIter {
            _phantom_map: PhantomData,
            has_next,
            next,
            get_key,
            get_value,
            iter,
        })
    }
}

/// An iterator over the keys and values in a map.
///
/// TODO: make the iterator implementation for java iterators its own thing
/// and generic enough to use elsewhere.
pub struct JMapIter<'map, 'a, 'b, O> {
    _phantom_map: PhantomData<&'map JMap<'a, O>>,
    has_next: JMethodID,
    next: JMethodID,
    get_key: JMethodID,
    get_value: JMethodID,
    iter: AutoLocal<'b, JObject<'b>>,
}

impl<'map, 'a, 'b, O> JMapIter<'map, 'a, 'b, O>
where
    O: AsRef<JObject<'a>>,
{
    pub fn next<'c>(&mut self, env: &mut JNIEnv<'c>) -> Result<Option<(JObject<'c>, JObject<'c>)>> {
        // SAFETY: We keep the class loaded, and fetched the method ID for these functions. We know none expect args.

        let iter = self.iter.as_obj();
        let has_next = unsafe {
            env.call_method_unchecked(
                iter,
                self.has_next,
                ReturnType::Primitive(Primitive::Boolean),
                &[],
            )
        }?
        .z()?;

        if !has_next {
            return Ok(None);
        }
        let next = unsafe {
            env.call_method_unchecked(iter, self.next, ReturnType::Object, &[])
        }?
        .l()?;
        let next = env.auto_local(next);

        let key = unsafe {
            env.call_method_unchecked(next, self.get_key, ReturnType::Object, &[])
        }?
        .l()?;

        let value = unsafe {
            env.call_method_unchecked(next, self.get_value, ReturnType::Object, &[])
        }?
        .l()?;

        Ok(Some((key, value)))
    }
}
