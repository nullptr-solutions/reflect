use core::{alloc::Layout, any::TypeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Field<'a> {
    pub ty_id: TypeId,

    pub name: &'a str,
    pub meta: FieldMeta<'a>,

    pub offset: usize,
    pub vis: Visibility,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Private,
    Pub,
    PubCrate,
    PubSuper,
    PubSelf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Struct<'a> {
    pub ty_id: TypeId,

    pub name: &'a str,
    pub meta: StructMeta<'a>,

    pub layout: Layout,
    pub vis: Visibility,

    pub fields: &'a [Field<'a>],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldMeta<'a> {
    pub meta: &'a [&'a str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructMeta<'a> {
    pub meta: &'a [&'a str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Value<'a> {
    pub ty_id: TypeId,

    pub name: &'a str,
    pub layout: Layout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type<'a> {
    Struct(Struct<'a>),
    Value(Value<'a>),
}

impl Type<'_> {
    pub fn id(&self) -> TypeId {
        match self {
            &Type::Struct(Struct { ty_id, .. }) => ty_id,
            &Type::Value(Value { ty_id, .. }) => ty_id,
        }
    }

    pub fn as_struct(&self) -> Option<&Struct> {
        match self {
            Type::Struct(x) => Some(x),
            _ => None,
        }
    }

    pub fn as_value(&self) -> Option<&Value> {
        match self {
            Type::Value(x) => Some(x),
            _ => None,
        }
    }
}

// traits

pub trait Reflect {
    const META: Type<'static>;
}

// introspection

#[cfg(windows)]
pub fn registered() -> &'static [&'static Type<'static>] {
    use core::slice;

    use crate::consts;

    #[repr(C)]
    struct DOS {
        magic: [u8; 2],
        pad: [u16; 28],
        e_lfanew: u32,
    }

    #[repr(C)]
    struct NT {
        magic: u32,
        machine: u16,
        nsects: u16,
        timestamp: u32,
        symtab: u32,
        symcount: u32,
        optsize: u16,
        characteristcs: u16,
    }

    #[repr(C)]
    #[derive(Debug)]
    struct Section {
        name: [u8; 8],
        vmsize: u32,
        vmaddr: u32,
        filesize: u32,
        fileoff: u32,
        relocsoff: u32,
        linesoff: u32,
        nrelocs: u16,
        nlines: u16,
        characteristcs: u32,
    }

    extern "C" {
        // Contains 'M' and 'Z' if the PE header is not wiped
        // The address of this constant is our own base address
        static __ImageBase: [u8; 2];
    }

    unsafe {
        let base = __ImageBase.as_ptr();

        // DOS Header, validate it.
        let dos = base.cast::<DOS>().read();
        assert_eq!(dos.magic, *b"MZ", "Invalid DOS Header");

        let ptr_nt = base.add(dos.e_lfanew as usize);

        // NT header, validate it.
        let nt = ptr_nt.cast::<NT>().read();
        assert_eq!(nt.magic, 0x4550, "Invalid NT Header");

        // Optional header, we'll trust it.
        let ptr_opt = ptr_nt.add(std::mem::size_of::<NT>());

        // Retrieve fields required to extract section data.
        let opt_size = nt.optsize; // Size of the optional header
        let num_sect = nt.nsects; // Number of sections

        // Retrieve the section table.
        let ptr_sect = ptr_opt.add(opt_size as usize);
        let sections = slice::from_raw_parts(ptr_sect.cast::<Section>(), num_sect as usize);

        // Retrieve the section we're interested in.
        let Some(sect) = sections.iter().find(|section| {
            let name_len = section.name.into_iter().position(|x| x == 0).unwrap_or(8);
            let name_slc = &section.name[..name_len];

            name_slc == consts::SECTNAME
        }) else {
            panic!("Failed to locate embedded reflection data.");
        };

        // Retrieve our target type's layout.
        let layout = Layout::new::<&Type>();

        // Ensure the section seems reasonable.
        assert_eq!(sect.vmsize as usize % layout.size(), 0);

        // Calculate entry count and address.
        let entries_len = sect.vmsize as usize / layout.size();
        let entries_ptr = base.add(sect.vmaddr as usize).cast::<&Type>();

        // Construct and return a slice containing all instances.
        slice::from_raw_parts(entries_ptr, entries_len)
    }
}

#[cfg(any(windows))]
#[cfg(feature = "std")]
pub fn registered_map() -> std::collections::HashMap<TypeId, Type<'static>> {
    let map: std::collections::HashMap<TypeId, Type> =
        registered().iter().map(|&x| (x.id(), x.clone())).collect();

    for ty in map.values() {
        if let Type::Struct(strukt) = ty {
            for field in strukt.fields {
                assert!(
                    map.contains_key(&field.ty_id),
                    "Field type not registered: {}->{}",
                    strukt.name,
                    field.name,
                );
            }
        }
    }

    map
}
