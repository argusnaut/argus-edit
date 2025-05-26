mod rustsyntaxhighlighter;
mod searchresulthighlighter;
mod syntaxhighlighter;

use rustsyntaxhighlighter::RustSyntaxHighlighter;
use searchresulthighlighter::SearchResultHighlighter;
use syntaxhighlighter::SyntaxHighlighter;

use crate::{
    editor::{annotation::Annotation, filetype::FileType, line::Line},
    prelude::{LineIndex, Location},
};

fn create_syntax_highlighter(filetype: FileType) -> Option<Box<dyn SyntaxHighlighter>> {
    match filetype {
        FileType::Rust => Some(Box::<RustSyntaxHighlighter>::default()),
        FileType::Text => None,
    }
}

#[derive(Default)]
pub struct Highlighter<'a> {
    syntax_highlighter: Option<Box<dyn SyntaxHighlighter>>,
    search_result_highlighter: Option<SearchResultHighlighter<'a>>,
}

impl<'a> Highlighter<'a> {
    pub fn new(
        matched_word: Option<&'a str>,
        selected_match: Option<Location>,
        filetype: FileType,
    ) -> Self {
        let search_result_highlighter = matched_word
            .map(|matched_word| SearchResultHighlighter::new(matched_word, selected_match));
        Self {
            syntax_highlighter: create_syntax_highlighter(filetype),
            search_result_highlighter,
        }
    }

    pub fn get_annotations(&self, index: LineIndex) -> Vec<Annotation> {
        let mut result = Vec::new();

        if let Some(syntax_highlighter) = &self.syntax_highlighter {
            if let Some(annotations) = syntax_highlighter.get_annotations(index) {
                result.extend(annotations.iter().copied());
            }
        }

        if let Some(search_result_highlighter) = &self.search_result_highlighter {
            if let Some(annotations) = search_result_highlighter.get_annotations(index) {
                result.extend(annotations.iter().copied());
            }
        }

        result
    }

    pub fn highlight(&mut self, index: LineIndex, line: &Line) {
        if let Some(syntax_highlighter) = &mut self.syntax_highlighter {
            syntax_highlighter.highlight(index, line);
        }

        if let Some(search_result_highlighter) = &mut self.search_result_highlighter {
            search_result_highlighter.highlight(index, line);
        }
    }
}
