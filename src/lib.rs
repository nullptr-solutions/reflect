#![feature(const_type_id, const_type_name)]

mod consts;
mod default;

pub mod macros;
pub mod meta;

#[cfg(test)]
mod tests {
    use std::{alloc::Layout, any::TypeId};

    use macro_rules_attribute::apply;

    use crate::{
        macros::reflect,
        meta::{Field, FieldMeta, Reflect, Struct, StructMeta, Type, Visibility},
    };

    #[test]
    fn struct_meta_matches_expected() {
        #[repr(C)]
        #[apply(reflect)]
        pub struct Bar;

        #[repr(C)]
        #[apply(reflect)]
        pub struct Foo {
            foo: u16,
            pub bar: u8,
            pub(super) baz: Bar,
            pub(crate) qux: Bar,
        }

        let fields = [
            Field {
                ty_id: TypeId::of::<u16>(),
                name: "foo",
                meta: FieldMeta { meta: &[] },
                offset: 0,
                vis: Visibility::Private,
            },
            Field {
                ty_id: TypeId::of::<u8>(),
                name: "bar",
                meta: FieldMeta { meta: &[] },
                offset: 2,
                vis: Visibility::Pub,
            },
            Field {
                ty_id: TypeId::of::<Bar>(),
                name: "baz",
                meta: FieldMeta { meta: &[] },
                offset: 3,
                vis: Visibility::PubSuper,
            },
            Field {
                ty_id: TypeId::of::<Bar>(),
                name: "qux",
                meta: FieldMeta { meta: &[] },
                offset: 3,
                vis: Visibility::PubCrate,
            },
        ];

        let expected = Type::Struct(Struct {
            ty_id: TypeId::of::<Foo>(),
            name: "reflect::tests::struct_meta_matches_expected::Foo",
            meta: StructMeta { meta: &["repr(C)"] },
            layout: Layout::from_size_align(4, 2).unwrap(),
            vis: Visibility::Pub,
            fields: &fields,
        });

        assert_eq!(Foo::META, expected);
    }
}
