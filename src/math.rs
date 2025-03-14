use nalgebra::{Matrix2, Matrix3, Matrix4, Vector2, Vector3, Vector4, Vector5};

#[cfg(not(feature = "f64"))]
/// Floating point type used by the library
pub type Fl = f32;
#[cfg(not(feature = "f64"))]
pub use std::f32 as std_fl;
#[cfg(feature = "f64")]
/// Floating point type used by the library
pub type Fl = f64;
#[cfg(feature = "f64")]
pub use std::f64 as std_fl;

/// Trait for numbers
pub trait IntoFl {
    /// Convert into a float
    fn into_fl(self) -> Fl;
}

macro_rules! impl_into_fl {
    ($ty:ident) => {
        impl IntoFl for $ty {
            fn into_fl(self) -> Fl {
                self as Fl
            }
        }
    };
}

impl_into_fl!(f32);
impl_into_fl!(f64);
impl_into_fl!(u8);
impl_into_fl!(i8);
impl_into_fl!(u16);
impl_into_fl!(i16);
impl_into_fl!(u32);
impl_into_fl!(i32);
impl_into_fl!(u64);
impl_into_fl!(i64);
impl_into_fl!(u128);
impl_into_fl!(i128);
impl_into_fl!(usize);
impl_into_fl!(isize);

/// A 2x2 matrix of floating point numbers
#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Mat2(pub Matrix2<Fl>);
/// A 3x3 matrix of floating point numbers
#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Mat3(pub Matrix3<Fl>);
/// A 4x4 matrix of floating point numbers
#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Mat4(pub Matrix4<Fl>);

macro_rules! impl_vec_n {
    ( $vec:ident[$len:expr], $inner:ident, $serde:ident=$serde_str:expr; $( $name:ident: $ty_name:ident ),* ) => {
        #[doc = concat!("A ", stringify!($len), "-dimensional vector of floating point numbers")]
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
        #[cfg_attr(
            feature = "serde",
            derive(::serde::Deserialize, ::serde::Serialize),
            serde(from = $serde_str, into = $serde_str)
        )]
        pub struct $vec(pub $inner<Fl>);

        impl $vec {
            #[inline(always)]
            /// Create a vector from a set of numbers
            pub fn new( $($name: impl IntoFl,)* ) -> Self {
                Self($inner::new( $($name.into_fl(),)* ))
            }
            $(
                #[inline(always)]
                #[doc = concat!("Access the ", stringify!($name), " component of this vector")]
                pub fn $name(&self) -> Fl {
                    self.0.$name
                }
            )*
        }
        impl<$($ty_name: IntoFl,)*> From<($($ty_name,)*)> for $vec {
            /// Convert from a tuple of numbers to a vector
            #[inline(always)]
            fn from(($($name,)*): ($($ty_name,)*)) -> Self {
                Self::new($($name,)*)
            }
        }
        impl From<$inner<Fl>> for $vec {
            /// Convert from an nalgebra vector
            #[inline(always)]
            fn from(inner: $inner<Fl>) -> Self {
                Self(inner)
            }
        }
        impl From<$vec> for $inner<Fl> {
            /// Convert from a care vector to an nalgebra vector
            #[inline(always)]
            fn from(vec: $vec) -> Self {
                vec.0
            }
        }
        impl std::ops::Deref for $vec {
            type Target = $inner<Fl>;

            #[inline(always)]
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl std::ops::DerefMut for $vec {
            #[inline(always)]
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl ::std::ops::Add<$vec> for $vec {
            type Output = $vec;

            #[inline(always)]
            /// Add two vectors (component-wise)
            fn add(self, rhs: Self) -> Self::Output {
                Self::new($(self.$name() + rhs.$name(),)*)
            }
        }

        impl ::std::ops::Sub<$vec> for $vec {
            type Output = $vec;

            #[inline(always)]
            /// Subtract two vectors (component-wise)
            fn sub(self, rhs: Self) -> Self::Output {
                Self::new($(self.$name() - rhs.$name(),)*)
            }
        }

        impl ::std::ops::Mul<$vec> for $vec {
            type Output = $vec;

            #[inline(always)]
            /// Multiply two vectors (component-wise)
            fn mul(self, rhs: Self) -> Self::Output {
                Self::new($(self.$name() * rhs.$name(),)*)
            }
        }

        impl ::std::ops::Mul<Fl> for $vec {
            type Output = $vec;

            #[inline(always)]
            /// Multiply a vector with a number
            fn mul(self, rhs: Fl) -> Self::Output {
                Self(self.0 * rhs)
            }
        }

        impl ::std::ops::Div<$vec> for $vec {
            type Output = $vec;

            #[inline(always)]
            /// Divide two vectors (component-wise)
            fn div(self, rhs: Self) -> Self::Output {
                Self::new($(self.$name() / rhs.$name(),)*)
            }
        }

        impl ::std::ops::Div<Fl> for $vec {
            type Output = $vec;

            #[inline(always)]
            /// Divide a vector by a number
            fn div(self, rhs: Fl) -> Self::Output {
                Self(self.0 / rhs)
            }
        }

        #[cfg(feature = "serde")]
        #[doc(hidden)]
        #[derive(Debug, ::serde::Deserialize, ::serde::Serialize)]
        struct $serde {
            $(
                $name: Fl,
            )*
        }

        #[cfg(feature = "serde")]
        impl From<$vec> for $serde {
            fn from(vec: $vec) -> Self {
                Self {
                    $(
                        $name: vec.$name,
                    )*
                }
            }
        }

        #[cfg(feature = "serde")]
        impl From<$serde> for $vec {
            fn from(vec: $serde) -> Self {
                Self::new(
                    $(
                        vec.$name,
                    )*
                )
            }
        }
    };
}

impl_vec_n!(Vec2[2], Vector2, SerdeVec2="SerdeVec2"; x: T, y: U);
impl_vec_n!(Vec3[2], Vector3, SerdeVec3="SerdeVec3"; x: T, y: U, z: V);
impl_vec_n!(Vec4[2], Vector4, SerdeVec4="SerdeVec4"; x: T, y: U, z: V, w: W);
impl_vec_n!(Vec5[2], Vector5, SerdeVec5="SerdeVec5"; x: T, y: U, z: V, w: W, a: S);

impl Mat4 {
    /// 4x4 identity matrix
    pub fn ident() -> Self {
        Mat4(Matrix4::identity())
    }
}

impl std::ops::Mul<Vec3> for &Mat4 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        Vec3((self.0 * Vector4::new(rhs.0.x, rhs.0.y, rhs.0.z, 1.0)).xyz())
    }
}

impl std::ops::Mul<Vec4> for &Mat4 {
    type Output = Vec4;

    fn mul(self, rhs: Vec4) -> Self::Output {
        Vec4(self.0 * rhs.0)
    }
}

impl Mat3 {
    /// 3x3 identity matrix
    pub fn ident() -> Self {
        Mat3(Matrix3::identity())
    }
}

impl std::ops::Mul<Vec2> for &Mat3 {
    type Output = Vec2;

    fn mul(self, rhs: Vec2) -> Self::Output {
        Vec2((self.0 * Vector3::new(rhs.0.x, rhs.0.y, 1.0)).xy())
    }
}

impl std::ops::Mul<Vec3> for &Mat3 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        Vec3(self.0 * rhs.0)
    }
}

impl Mat2 {
    /// 2x2 identity matrix
    pub fn ident() -> Self {
        Mat2(Matrix2::identity())
    }
    /// Create a new matrix from the 4 components, column major
    pub fn new(x1: impl IntoFl, y1: impl IntoFl, x2: impl IntoFl, y2: impl IntoFl) -> Self {
        Mat2(Matrix2::new(
            x1.into_fl(),
            y1.into_fl(),
            x2.into_fl(),
            y2.into_fl(),
        ))
    }
}

impl Vec2 {
    #[inline]
    /// Return a version of this vector that has been rotated by `rotation` radians clockwise
    pub fn rotated(&self, rotation: Fl) -> Self {
        let (s, c) = (rotation.sin(), rotation.cos());
        Self::new(self.0.x * c + self.0.y * s, self.0.y * c - self.0.x * s)
    }
    /// Return a version of this vector that's been rotated by 90 degrees clockwise
    pub fn tangent(&self) -> Self {
        Self::new(self.0.y, -self.0.x)
    }
    /// Return the euclidian length (l1 norm) of this vector
    pub fn length(&self) -> Fl {
        (self.0.x.powi(2) + self.0.y.powi(2)).sqrt()
    }
    /// Return the euclidian length (l1 norm) of this vector
    pub fn normalize_or(&self, other: Vec2) -> Self {
        if self.length() <= 0.000001 {
            other
        } else {
            *self / self.length()
        }
    }
    /// Normalize this vector, or return zero for a zero vector
    pub fn normalize(&self) -> Self {
        self.normalize_or(*self)
    }
}

impl std::ops::Mul<Vec2> for &Mat2 {
    type Output = Vec2;

    fn mul(self, rhs: Vec2) -> Self::Output {
        Vec2(self.0 * rhs.0)
    }
}

/// Good set of default imports
pub mod prelude {
    pub use super::Fl;
    pub use super::Mat2;
    pub use super::Mat3;
    pub use super::Mat4;
    pub use super::Vec2;
    pub use super::Vec3;
    pub use super::Vec4;
}
