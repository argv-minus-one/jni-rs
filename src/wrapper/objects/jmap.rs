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
pub struct JMap<'local: 'obj_ref, 'obj_ref> {
    internal: &'obj_ref JObject<'local>,
    class: AutoLocal<'local, JClass<'local>>,
    get: JMethodID,
    put: JMethodID,
    remove: JMethodID,
}

impl<'local: 'obj_ref, 'obj_ref> AsRef<JMap<'local, 'obj_ref>> for JMap<'local, 'obj_ref> {
    fn as_ref(&self) -> &JMap<'local, 'obj_ref> {
        self
    }
}

impl<'local: 'obj_ref, 'obj_ref> AsRef<JObject<'local>> for JMap<'local, 'obj_ref> {
    fn as_ref(&self) -> &JObject<'local> {
        self.internal
    }
}

impl<'local: 'obj_ref, 'obj_ref> JMap<'local, 'obj_ref> {
    /// Create a map from the environment and an object. This looks up the
    /// necessary class and method ids to call all of the methods on it so that
    /// exra work doesn't need to be done on every method call.
    pub fn from_env(env: &mut JNIEnv<'local>, obj: &'obj_ref JObject<'local>) -> Result<JMap<'local, 'obj_ref>> {
        let class = AutoLocal::new(env.find_class("java/util/Map")?, env);

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
    pub fn get<'other_local>(&self, env: &mut JNIEnv<'other_local>, key: &JObject) -> Result<Option<JObject<'other_local>>> {
        // SAFETY: We keep the class loaded, and fetched the method ID for this function.
        // Provided argument is statically known as a JObject/null, rather than another primitive type.
        let result = unsafe {
            env.call_method_unchecked(
                self.internal,
                self.get,
                ReturnType::Object,
                &[JValue::from(key).as_jni()],
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
    pub fn put<'other_local>(&self, env: &mut JNIEnv<'other_local>, key: &JObject, value: &JObject) -> Result<Option<JObject<'other_local>>> {
        // SAFETY: We keep the class loaded, and fetched the method ID for this function.
        // Provided argument is statically known as a JObject/null, rather than another primitive type.
        let result = unsafe {
            env.call_method_unchecked(
                self.internal,
                self.put,
                ReturnType::Object,
                &[JValue::from(key).as_jni(), JValue::from(value).as_jni()],
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
    pub fn remove<'other_local>(&self, env: &mut JNIEnv<'other_local>, key: &JObject) -> Result<Option<JObject<'other_local>>> {
        // SAFETY: We keep the class loaded, and fetched the method ID for this function.
        // Provided argument is statically known as a JObject/null, rather than another primitive type.
        let result = unsafe {
            env.call_method_unchecked(
                self.internal,
                self.remove,
                ReturnType::Object,
                &[JValue::from(key).as_jni()],
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
    pub fn iter<'map, 'iter_local>(&'map self, env: &mut JNIEnv<'iter_local>) -> Result<JMapIter<'map, 'local, 'obj_ref, 'iter_local>> {
        let iter_class = AutoLocal::new(env.find_class("java/util/Iterator")?, env);

        let has_next = env.get_method_id(&iter_class, "hasNext", "()Z")?;

        let next = env
            .get_method_id(&iter_class, "next", "()Ljava/lang/Object;")?;

        let entry_class = AutoLocal::new(env.find_class("java/util/Map$Entry")?, env);

        let get_key = env
            .get_method_id(&entry_class, "getKey", "()Ljava/lang/Object;")?;

        let get_value = env
            .get_method_id(&entry_class, "getValue", "()Ljava/lang/Object;")?;

        // Get the iterator over Map entries.
        // Use the local frame till #109 is resolved, so that implicitly looked-up
        // classes are freed promptly.
        let iter = env.with_local_frame(16, |env| {
            // SAFETY: We keep the class loaded, and fetched the method ID for this function. Arg list is known empty.
            let entry_set = unsafe {
                env.call_method_unchecked(
                    self.internal,
                    (&self.class, "entrySet", "()Ljava/util/Set;"),
                    ReturnType::Object,
                    &[],
                )
            }?
            .l()?;

            // SAFETY: We keep the class loaded, and fetched the method ID for this function. Arg list is known empty.
            let iter = unsafe {
                env.call_method_unchecked(
                    entry_set,
                    ("java/util/Set", "iterator", "()Ljava/util/Iterator;"),
                    ReturnType::Object,
                    &[],
                )
            }?
            .l()?;

            Ok(iter)
        })?;
        let iter = env.auto_local(iter);

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
pub struct JMapIter<'map, 'local: 'obj_ref, 'obj_ref, 'iter_local> {
    _phantom_map: PhantomData<&'map JMap<'local, 'obj_ref>>,
    has_next: JMethodID,
    next: JMethodID,
    get_key: JMethodID,
    get_value: JMethodID,
    iter: AutoLocal<'iter_local, JObject<'iter_local>>,
}

impl<'map, 'local: 'obj_ref, 'obj_ref, 'iter_local> JMapIter<'map, 'local, 'obj_ref, 'iter_local> {
    /// Advances the iterator and returns the next key-value pair in the
    /// `java.util.Map`, or `None` if there are no more objects.
    ///
    /// This returns:
    ///
    /// * `Ok(Some(_))`: if there was another key-value pair in the map.
    /// * `Ok(None)`: if there are no more key-value pairs in the map.
    /// * `Err(_)`: if there was an error calling the Java method to
    ///   get the next key-value pair.
    ///
    /// This is like [`Iterator::next`], but requires a parameter of
    /// type `&mut JNIEnv` in order to call into Java.
    pub fn next<'other_local>(&mut self, env: &mut JNIEnv<'other_local>) -> Result<Option<(JObject<'other_local>, JObject<'other_local>)>> {
        // SAFETY: We keep the class loaded, and fetched the method ID for these functions. We know none expect args.

        let has_next = unsafe {
            env.call_method_unchecked(
                &self.iter,
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
            env.call_method_unchecked(&self.iter, self.next, ReturnType::Object, &[])
        }?
        .l()?;
        let next = env.auto_local(next);

        let key = unsafe {
            env.call_method_unchecked(&next, self.get_key, ReturnType::Object, &[])
        }?
        .l()?;

        let value = unsafe {
            env.call_method_unchecked(&next, self.get_value, ReturnType::Object, &[])
        }?
        .l()?;

        Ok(Some((key, value)))
    }
}
