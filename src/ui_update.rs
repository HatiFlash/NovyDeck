//! Update and offline release-notes presentation.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum NoteBlock { Heading(String), Bullet { title: String, body: String }, Paragraph(String) }

pub(super) fn parse_notes(_markdown: &str, _lang: &str) -> Vec<NoteBlock> { Vec::new() }
