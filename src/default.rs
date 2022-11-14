use core::{alloc::Layout, any::TypeId};

use crate::{
    macros,
    meta::{Reflect, Type, Value},
};

macro_rules! register_value {
    ($($ty:ty),*) => {$(
        impl Reflect for $ty {
            const META: Type<'static> = Type::Value(Value {
                ty_id: TypeId::of::<$ty>(),
                name: std::any::type_name::<$ty>(),
                layout: Layout::new::<$ty>(),
            });
        }

        macros::register!($ty);
    )*};
}

register_value!(u8, u16, u32, u64, u128);
register_value!(i8, i16, i32, i64, i128);
