use std::mem::size_of;

pub const SECTION: &str = ".note.decl";

macro_rules! infer_array_size {
    (pub const $name:ident: [$ty:ty; _] = $val:expr;) => {
        pub const $name: [$ty; $val.len()] = $val;
    };
}

infer_array_size! {
    pub const NAME: [u8; _] = *b"decl";
}

#[allow(clippy::module_name_repetitions)]
#[repr(u32)]
pub enum NoteType {
    Version = 1,
    Data = 3,
}

#[repr(C, packed(4))]
struct Packed<T>(T);

#[repr(C, align(4))]
struct Align<T>(T);

#[allow(clippy::module_name_repetitions, dead_code)]
#[repr(C, align(4))]
pub struct Note<T> {
    namesz: u32,
    descsz: u32,
    r#type: u32,
    name: [u8; NAME.len()],
    desc: Align<Packed<T>>,
}

impl<T> Note<T> {
    #[allow(clippy::cast_possible_truncation)]
    pub const fn new(r#type: NoteType, desc: T) -> Self {
        Note {
            namesz: NAME.len() as u32,
            descsz: size_of::<T>() as u32,
            r#type: r#type as u32,
            name: NAME,
            desc: Align(Packed(desc)),
        }
    }
}
