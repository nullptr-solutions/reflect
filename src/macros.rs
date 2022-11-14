#[doc(hidden)]
pub use memoffset::offset_of as _offset_of;

#[doc(hidden)]
#[macro_export]
macro_rules! _vis_to_enum {
    ($vis:vis) => {{
        match stringify!($vis).as_bytes() {
            b"" => $crate::meta::Visibility::Private,
            b"pub " => $crate::meta::Visibility::Pub,
            b"pub(crate) " | b"pub(in crate) " => $crate::meta::Visibility::PubCrate,
            b"pub(super) " | b"pub(in super) " => $crate::meta::Visibility::PubSuper,
            b"pub(self) " | b"pub(in self) " => $crate::meta::Visibility::PubSelf,
            _ => panic!(concat!("unknown visibility: `", stringify!($vis), "`")),
        }
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! _field_meta {
    ($($meta:meta),*) => {{
        $crate::meta::FieldMeta {
            meta: &[$(stringify!($meta)),*]
        }
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! _struct_meta {
    ($($meta:meta),*) => {{
        $crate::meta::StructMeta {
            meta: &[$(stringify!($meta)),*]
        }
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! _field {
    ($struct:path, $vis:vis, $name:ident, $typ:ty, $($meta:meta),*) => {{
        $crate::meta::Field {
            name: stringify!($name),
            meta: $crate::macros::_field_meta!($($meta),*),
            ty_id: ::std::any::TypeId::of::<$typ>(),
            offset: $crate::macros::_offset_of!($struct, $name),
            vis: $crate::macros::_vis_to_enum!($vis),
        }
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! _struct {
    ($vis:vis, $struct:path, $($meta:meta),*) => {{
        $crate::meta::Struct {
            ty_id: ::std::any::TypeId::of::<$struct>(),
            meta: $crate::macros::_struct_meta!($($meta),*),
            name: ::std::any::type_name::<$struct>(),
            layout: ::std::alloc::Layout::new::<$struct>(),
            vis: $crate::macros::_vis_to_enum!($vis),
            fields: FIELDS,
        }
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! _assert_registered {
    ($($meta_ty:ty),*) => {
        const _: () = {
            use $crate::meta::Reflect;

            const fn trait_check<T: Reflect>() {}

            $( trait_check::<$meta_ty>(); )*
        };
    };
}

#[macro_export]
macro_rules! register {
    ($meta_ty:ty) => {
        const _: () = {
            #[used]
            #[link_section = ".memex"] // MUST MATCH SECTNAME IN CONSTS
            static _REGISTER: &'static $crate::meta::Type =
                &<$meta_ty as $crate::meta::Reflect>::META;
        };

        $crate::macros::_assert_registered!($meta_ty);
    };
}

#[macro_export]
macro_rules! reflect {
    // struct Foo { bar: T, baz: U }
    (
        $( #[$meta:meta] )*
        $vis:vis struct $name:ident {
            $(
                $( #[$field_meta:meta] )*
                $field_vis:vis $field_name:ident : $field_ty:ty
            ),*
        $(,)? }
    ) => {
        $( #[$meta] )*
        $vis struct $name {
            $(
                $( #[$field_meta] )*
                $field_vis $field_name : $field_ty
            ),*
        }

        impl $crate::meta::Reflect for $name {
            const META: $crate::meta::Type<'static> = $crate::meta::Type::Struct({{
                const STRUCT: $crate::meta::Struct = $crate::macros::_struct!($vis, $name, $($meta),*);
                const FIELDS: &'static [$crate::meta::Field] = &[
                    $($crate::macros::_field!($name, $field_vis, $field_name, $field_ty, $($field_meta),*)),*
                ];

                STRUCT
            }});
        }

        $crate::macros::_assert_registered!($($field_ty),*);
        $crate::macros::register!($name);
    };

    // struct Foo;
    (
        $( #[$meta:meta] )*
        $vis:vis struct $name:ident;
    ) => {

        $( #[$meta] )*
        $vis struct $name;

        impl $crate::meta::Reflect for $name {
            const META: $crate::meta::Type<'static> = $crate::meta::Type::Struct({{
                const STRUCT: $crate::meta::Struct = $crate::macros::_struct!($vis, $name, $($meta),*);
                const FIELDS: &'static [$crate::meta::Field] = &[];

                STRUCT
            }});
        }

        $crate::macros::register!($name);
    };
}

// Private API
pub use _assert_registered;
pub use _field;
pub use _field_meta;
pub use _struct;
pub use _struct_meta;
pub use _vis_to_enum;
// Public API
pub use reflect;
pub use register;
