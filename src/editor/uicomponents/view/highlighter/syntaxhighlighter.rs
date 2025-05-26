use crate::{
    editor::{annotation::Annotation, line::Line},
    prelude::LineIndex,
};

pub trait SyntaxHighlighter {
    fn highlight(&mut self, index: LineIndex, line: &Line);
    fn get_annotations(&self, index: LineIndex) -> Option<&Vec<Annotation>>;
}
