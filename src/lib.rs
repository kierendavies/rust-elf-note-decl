use std::{
    ffi::{CStr, FromBytesWithNulError},
    str,
};

use decl_model::{
    note::{Type as NoteType, SECTION},
    Data,
};
use object::{
    elf::{FileHeader32, FileHeader64},
    read::elf::{FileHeader, Note, SectionHeader},
    Endianness, FileKind,
};

pub use decl_macros::data;
pub use decl_model as model;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("invalid note name: `{0}`")]
    InvalidNoteName(String),

    #[error("invalid note type: {0:#010x}")]
    InvalidNoteType(u32),

    #[error("invalid ELF section type: `{0:#010x}`")]
    InvalidSectionType(u32),

    #[error("parsing JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("missing note: {:#010x} ({0:?})", *.0 as u32)]
    MissingNote(NoteType),

    #[error("missing ELF section: `{SECTION}`")]
    MissingSection,

    #[error("parsing C string: {0}")]
    CStr(#[from] FromBytesWithNulError),

    #[error("reading object file: {0}")]
    ReadObject(#[from] object::read::Error),

    #[error("unsupported file kind: {0:?}")]
    UnsupportedFileKind(FileKind),

    #[error("unsupported data version: {0}")]
    UnsupportedVersion(String),

    #[error("invalid utf-8: {0}")]
    Utf8(#[from] str::Utf8Error),
}

pub fn parse(file_contents: &[u8]) -> Result<Data, ParseError> {
    let kind = object::FileKind::parse(file_contents)?;

    match kind {
        object::FileKind::Elf32 => parse_kind::<FileHeader32<Endianness>>(file_contents),
        object::FileKind::Elf64 => parse_kind::<FileHeader64<Endianness>>(file_contents),
        _ => unimplemented!(),
    }
}

fn parse_kind<Elf: FileHeader>(file_contents: &[u8]) -> Result<Data, ParseError> {
    let elf = Elf::parse(file_contents)?;
    let endian = elf.endian()?;

    let notes = parse_notes(elf, file_contents)?;

    let version = parse_version(endian, &notes)?;

    match version {
        "0.1.0" => parse_data_0_1_0(endian, &notes),
        _ => Err(ParseError::UnsupportedVersion(version.to_string())),
    }
}

fn parse_notes<'a, Elf: FileHeader>(
    elf: &Elf,
    file_contents: &'a [u8],
) -> Result<Vec<Note<'a, Elf>>, ParseError> {
    let endian = elf.endian()?;

    let (_, section) = elf
        .sections(endian, file_contents)?
        .section_by_name(endian, SECTION.as_bytes())
        .ok_or(ParseError::MissingSection)?;

    let note_name_without_nul = CStr::from_bytes_with_nul(&decl_model::note::NAME)
        .unwrap()
        .to_bytes();

    let mut notes_iter = section
        .notes(endian, file_contents)?
        .ok_or_else(|| ParseError::InvalidSectionType(section.sh_type(endian)))?;
    let mut notes = Vec::new();
    while let Some(note) = notes_iter.next()? {
        if note.name() != note_name_without_nul {
            continue;
        }
        notes.push(note);
    }

    Ok(notes)
}

fn parse_version<'a, Elf: FileHeader>(
    endian: Elf::Endian,
    notes: &[Note<'a, Elf>],
) -> Result<&'a str, ParseError> {
    let version_note = find_note(endian, notes, NoteType::Version)?;

    let version = CStr::from_bytes_with_nul(version_note.desc())?.to_str()?;

    Ok(version)
}

fn parse_data_0_1_0<Elf: FileHeader>(
    endian: Elf::Endian,
    notes: &[Note<Elf>],
) -> Result<Data, ParseError> {
    let data_note = find_note(endian, notes, NoteType::Data)?;

    let data_bytes = CStr::from_bytes_with_nul(data_note.desc())?.to_bytes();
    let data = serde_json::from_slice(data_bytes)?;

    Ok(data)
}

fn find_note<'a, 'b, Elf: FileHeader>(
    endian: Elf::Endian,
    notes: &'b [Note<'a, Elf>],
    note_type: NoteType,
) -> Result<&'b Note<'a, Elf>, ParseError> {
    notes
        .iter()
        .find(|note| note.n_type(endian) == note_type as u32)
        .ok_or(ParseError::MissingNote(note_type))
}
