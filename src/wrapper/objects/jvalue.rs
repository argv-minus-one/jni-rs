use std::convert::TryFrom;
use std::fmt::Debug;
use std::mem::transmute;

use log::trace;

use crate::{errors::*, objects::JObject, signature::Primitive, sys::*};

/// Rusty version of the JNI C `jvalue` enum. Used in Java method call arguments
/// and returns.
///
/// `JValue` is a generic type, meant to represent both owned and borrowed JNI
/// values. The type parameter `O` refers to what kind of object reference the
/// `JValue` can hold, which is either:
///
/// * an owned [`JObject`], used for values returned from a Java method call,
///   or
/// * a borrowed `&JObject`, used for parameters passed to a Java method call.
///
/// These two cases are represented by the type aliases [`JValueOwned`] and
/// [`JValueRef`], respectively.
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug)]
pub enum JValue<O> {
    Object(O),
    Byte(jbyte),
    Char(jchar),
    Short(jshort),
    Int(jint),
    Long(jlong),
    Bool(jboolean),
    Float(jfloat),
    Double(jdouble),
    Void,
}

/// An <dfn>owned</dfn> [`JValue`].
///
/// This type is used for values returned from Java method calls. If the Java
/// method returns an object reference, it will take the form of an owned
/// [`JObject`].
pub type JValueOwned<'a> = JValue<JObject<'a>>;

/// A <dfn>reference</dfn> [`JValue`].
///
/// This type is used for parameters passed to Java method calls. If the Java
/// method is to be passed an object reference, it takes the form of a borrowed
/// <code>&[JObject]</code>.
pub type JValueRef<'a: 'b, 'b> = JValue<&'b JObject<'a>>;

impl<O> JValue<O> {
    /// Convert the enum to its jni-compatible equivalent.
    pub fn as_jni<'a>(&self) -> jvalue
    where
        O: AsRef<JObject<'a>> + Debug,
    {
        let val: jvalue = match self {
            JValue::Object(obj) => jvalue {
                l: unsafe { transmute(obj) },
            },
            JValue::Byte(byte) => jvalue { b: *byte },
            JValue::Char(char) => jvalue { c: *char },
            JValue::Short(short) => jvalue { s: *short },
            JValue::Int(int) => jvalue { i: *int },
            JValue::Long(long) => jvalue { j: *long },
            JValue::Bool(boolean) => jvalue { b: *boolean as i8 },
            JValue::Float(float) => jvalue { f: *float },
            JValue::Double(double) => jvalue { d: *double },
            JValue::Void => jvalue {
                l: ::std::ptr::null_mut(),
            },
        };
        trace!("converted {:?} to jvalue {:?}", self, unsafe {
            ::std::mem::transmute::<_, u64>(val)
        });
        val
    }

    /// Get the type name for the enum variant.
    pub fn type_name(&self) -> &'static str {
        match *self {
            JValue::Void => "void",
            JValue::Object(_) => "object",
            JValue::Byte(_) => "byte",
            JValue::Char(_) => "char",
            JValue::Short(_) => "short",
            JValue::Int(_) => "int",
            JValue::Long(_) => "long",
            JValue::Bool(_) => "bool",
            JValue::Float(_) => "float",
            JValue::Double(_) => "double",
        }
    }

    /// Get the primitive type for the enum variant. If it's not a primitive
    /// (i.e. an Object), returns None.
    pub fn primitive_type(&self) -> Option<Primitive> {
        Some(match *self {
            JValue::Object(_) => return None,
            JValue::Void => Primitive::Void,
            JValue::Byte(_) => Primitive::Byte,
            JValue::Char(_) => Primitive::Char,
            JValue::Short(_) => Primitive::Short,
            JValue::Int(_) => Primitive::Int,
            JValue::Long(_) => Primitive::Long,
            JValue::Bool(_) => Primitive::Boolean,
            JValue::Float(_) => Primitive::Float,
            JValue::Double(_) => Primitive::Double,
        })
    }

    /// Try to unwrap to an Object.
    pub fn l(self) -> Result<O> {
        match self {
            JValue::Object(obj) => Ok(obj),
            _ => Err(Error::WrongJValueType("object", self.type_name())),
        }
    }

    /// Try to unwrap to a boolean.
    pub fn z(self) -> Result<bool> {
        match self {
            JValue::Bool(b) => Ok(b == JNI_TRUE),
            _ => Err(Error::WrongJValueType("bool", self.type_name())),
        }
    }

    /// Try to unwrap to a byte.
    pub fn b(self) -> Result<jbyte> {
        match self {
            JValue::Byte(b) => Ok(b),
            _ => Err(Error::WrongJValueType("jbyte", self.type_name())),
        }
    }

    /// Try to unwrap to a char.
    pub fn c(self) -> Result<jchar> {
        match self {
            JValue::Char(b) => Ok(b),
            _ => Err(Error::WrongJValueType("jchar", self.type_name())),
        }
    }

    /// Try to unwrap to a double.
    pub fn d(self) -> Result<jdouble> {
        match self {
            JValue::Double(b) => Ok(b),
            _ => Err(Error::WrongJValueType("jdouble", self.type_name())),
        }
    }

    /// Try to unwrap to a float.
    pub fn f(self) -> Result<jfloat> {
        match self {
            JValue::Float(b) => Ok(b),
            _ => Err(Error::WrongJValueType("jfloat", self.type_name())),
        }
    }

    /// Try to unwrap to an int.
    pub fn i(self) -> Result<jint> {
        match self {
            JValue::Int(b) => Ok(b),
            _ => Err(Error::WrongJValueType("jint", self.type_name())),
        }
    }

    /// Try to unwrap to a long.
    pub fn j(self) -> Result<jlong> {
        match self {
            JValue::Long(b) => Ok(b),
            _ => Err(Error::WrongJValueType("jlong", self.type_name())),
        }
    }

    /// Try to unwrap to a short.
    pub fn s(self) -> Result<jshort> {
        match self {
            JValue::Short(b) => Ok(b),
            _ => Err(Error::WrongJValueType("jshort", self.type_name())),
        }
    }

    /// Try to unwrap to a void.
    pub fn v(self) -> Result<()> {
        match self {
            JValue::Void => Ok(()),
            _ => Err(Error::WrongJValueType("void", self.type_name())),
        }
    }

    // Preserve the table formatting in the next function.
    //
    // I'd prefer to apply this attribute only to the `match` expression
    // inside, but non-built-in attributes on expressions are currently
    // experimental, so I can't.
    #[rustfmt::skip]
    /// Copies or borrows the value in this `JValue`.
    ///
    /// If the value is a primitive type, it is copied. If the value is an
    /// object reference, it is borrowed.
    pub fn borrow<'a>(&'a self) -> JValue<&'a O> {
        match self {
            JValue::Object(o) => JValue::Object(o ),
            JValue::Byte  (v) => JValue::Byte  (*v),
            JValue::Char  (v) => JValue::Char  (*v),
            JValue::Short (v) => JValue::Short (*v),
            JValue::Int   (v) => JValue::Int   (*v),
            JValue::Long  (v) => JValue::Long  (*v),
            JValue::Bool  (v) => JValue::Bool  (*v),
            JValue::Float (v) => JValue::Float (*v),
            JValue::Double(v) => JValue::Double(*v),
            JValue::Void      => JValue::Void      ,
        }
    }
}

impl<'a, O> From<&'a JValue<O>> for JValue<&'a O> {
    fn from(other: &'a JValue<O>) -> Self {
        other.borrow()
    }
}

impl<'a, T: Into<JObject<'a>>> From<T> for JValueOwned<'a> {
    fn from(other: T) -> Self {
        Self::Object(other.into())
    }
}

impl<'a: 'b, 'b, T: AsRef<JObject<'a>>> From<&'b T> for JValueRef<'a, 'b> {
    fn from(other: &'b T) -> Self {
        Self::Object(other.as_ref())
    }
}

impl<'a> TryFrom<JValueOwned<'a>> for JObject<'a> {
    type Error = Error;

    fn try_from(value: JValueOwned<'a>) -> Result<Self> {
        match value {
            JValue::Object(o) => Ok(o),
            _ => Err(Error::WrongJValueType("object", value.type_name())),
        }
    }
}

impl<O> From<bool> for JValue<O> {
    fn from(other: bool) -> Self {
        JValue::Bool(if other { JNI_TRUE } else { JNI_FALSE })
    }
}

// jbool
impl<O> From<jboolean> for JValue<O> {
    fn from(other: jboolean) -> Self {
        JValue::Bool(other)
    }
}

impl<O> TryFrom<JValue<O>> for jboolean {
    type Error = Error;

    fn try_from(value: JValue<O>) -> Result<Self> {
        match value {
            JValue::Bool(b) => Ok(b),
            _ => Err(Error::WrongJValueType("bool", value.type_name())),
        }
    }
}

// jchar
impl<O> From<jchar> for JValue<O> {
    fn from(other: jchar) -> Self {
        JValue::Char(other)
    }
}

impl<O> TryFrom<JValue<O>> for jchar {
    type Error = Error;

    fn try_from(value: JValue<O>) -> Result<Self> {
        match value {
            JValue::Char(c) => Ok(c),
            _ => Err(Error::WrongJValueType("char", value.type_name())),
        }
    }
}

// jshort
impl<O> From<jshort> for JValue<O> {
    fn from(other: jshort) -> Self {
        JValue::Short(other)
    }
}

impl<O> TryFrom<JValue<O>> for jshort {
    type Error = Error;

    fn try_from(value: JValue<O>) -> Result<Self> {
        match value {
            JValue::Short(s) => Ok(s),
            _ => Err(Error::WrongJValueType("short", value.type_name())),
        }
    }
}

// jfloat
impl<O> From<jfloat> for JValue<O> {
    fn from(other: jfloat) -> Self {
        JValue::Float(other)
    }
}

impl<O> TryFrom<JValue<O>> for jfloat {
    type Error = Error;

    fn try_from(value: JValue<O>) -> Result<Self> {
        match value {
            JValue::Float(f) => Ok(f),
            _ => Err(Error::WrongJValueType("float", value.type_name())),
        }
    }
}

// jdouble
impl<O> From<jdouble> for JValue<O> {
    fn from(other: jdouble) -> Self {
        JValue::Double(other)
    }
}

impl<O> TryFrom<JValue<O>> for jdouble {
    type Error = Error;

    fn try_from(value: JValue<O>) -> Result<Self> {
        match value {
            JValue::Double(d) => Ok(d),
            _ => Err(Error::WrongJValueType("double", value.type_name())),
        }
    }
}

// jint
impl<O> From<jint> for JValue<O> {
    fn from(other: jint) -> Self {
        JValue::Int(other)
    }
}

impl<O> TryFrom<JValue<O>> for jint {
    type Error = Error;

    fn try_from(value: JValue<O>) -> Result<Self> {
        match value {
            JValue::Int(i) => Ok(i),
            _ => Err(Error::WrongJValueType("int", value.type_name())),
        }
    }
}

// jlong
impl<O> From<jlong> for JValue<O> {
    fn from(other: jlong) -> Self {
        JValue::Long(other)
    }
}

impl<O> TryFrom<JValue<O>> for jlong {
    type Error = Error;

    fn try_from(value: JValue<O>) -> Result<Self> {
        match value {
            JValue::Long(l) => Ok(l),
            _ => Err(Error::WrongJValueType("long", value.type_name())),
        }
    }
}

// jbyte
impl<O> From<jbyte> for JValue<O> {
    fn from(other: jbyte) -> Self {
        JValue::Byte(other)
    }
}

impl<O> TryFrom<JValue<O>> for jbyte {
    type Error = Error;

    fn try_from(value: JValue<O>) -> Result<Self> {
        match value {
            JValue::Byte(b) => Ok(b),
            _ => Err(Error::WrongJValueType("byte", value.type_name())),
        }
    }
}

// jvoid
impl<O> From<()> for JValue<O> {
    fn from(_: ()) -> Self {
        JValue::Void
    }
}

impl<O> TryFrom<JValue<O>> for () {
    type Error = Error;

    fn try_from(value: JValue<O>) -> Result<Self> {
        match value {
            JValue::Void => Ok(()),
            _ => Err(Error::WrongJValueType("void", value.type_name())),
        }
    }
}
