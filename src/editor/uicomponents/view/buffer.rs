use crate::editor::annotatedstring::AnnotatedString;
use crate::prelude::*;

use std::fs::{File, read_to_string};
use std::io::{Error, Write};
use std::ops::Range;

use super::FileInfo;
use super::Line;
use super::highlighter::Highlighter;

#[derive(Default)]
pub struct Buffer {
    lines: Vec<Line>,
    fileinfo: FileInfo,
    dirty: bool,
}

impl Buffer {
    pub const fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub const fn get_fileinfo(&self) -> &FileInfo {
        &self.fileinfo
    }

    pub fn grapheme_count(&self, index: LineIndex) -> GraphemeIndex {
        self.lines.get(index).map_or(0, Line::grapheme_count)
    }

    pub fn width_until(&self, index: LineIndex, until: GraphemeIndex) -> GraphemeIndex {
        self.lines
            .get(index)
            .map_or(0, |line| line.width_until(until))
    }

    pub fn get_highlighted_substring(
        &self,
        line_index: LineIndex,
        range: Range<GraphemeIndex>,
        highlighter: &Highlighter,
    ) -> Option<AnnotatedString> {
        self.lines.get(line_index).map(|line| {
            line.get_annotated_visible_substr(range, Some(&highlighter.get_annotations(line_index)))
        })
    }

    pub fn highlight(&self, index: LineIndex, highlighter: &mut Highlighter) {
        if let Some(line) = self.lines.get(index) {
            highlighter.highlight(index, line);
        }
    }

    pub fn load(filename: &str) -> Result<Self, Error> {
        let contents = read_to_string(filename)?;
        let mut lines = Vec::new();
        for value in contents.lines() {
            lines.push(Line::from(value));
        }
        Ok(Self {
            lines,
            fileinfo: FileInfo::from(filename),
            dirty: false,
        })
    }

    pub fn search_forward(&self, query: &str, from: Location) -> Option<Location> {
        if query.is_empty() {
            return None;
        }

        let mut is_first = true;
        for (line_index, line) in self
            .lines
            .iter()
            .enumerate()
            .cycle()
            .skip(from.line_index)
            .take(self.lines.len().saturating_add(1))
        {
            let from_grapheme_index = if is_first {
                is_first = false;
                from.grapheme_index
            } else {
                0
            };

            if let Some(grapheme_index) = line.search_forward(query, from_grapheme_index) {
                return Some(Location {
                    grapheme_index,
                    line_index,
                });
            }
        }

        None
    }

    pub fn search_backward(&self, query: &str, from: Location) -> Option<Location> {
        if query.is_empty() {
            return None;
        }

        let mut is_first = true;
        for (line_index, line) in self
            .lines
            .iter()
            .enumerate()
            .rev()
            .cycle()
            .skip(
                self.lines
                    .len()
                    .saturating_sub(from.line_index)
                    .saturating_sub(1),
            )
            .take(self.lines.len().saturating_add(1))
        {
            let from_grapheme_index = if is_first {
                is_first = false;
                from.grapheme_index
            } else {
                line.grapheme_count()
            };

            if let Some(grapheme_index) = line.search_backward(query, from_grapheme_index) {
                return Some(Location {
                    grapheme_index,
                    line_index,
                });
            }
        }

        None
    }

    pub fn save_to_file(&self, fileinfo: &FileInfo) -> Result<(), Error> {
        if let Some(path) = &fileinfo.get_path() {
            let mut file = File::create(path)?;
            for line in &self.lines {
                writeln!(file, "{line}")?;
            }
        } else {
            #[cfg(debug_assertions)]
            {
                panic!("Attempting to save with no file path present");
            }
        }
        Ok(())
    }

    pub fn save_as(&mut self, filename: &str) -> Result<(), Error> {
        let fileinfo = FileInfo::from(filename);
        self.save_to_file(&fileinfo)?;
        self.fileinfo = fileinfo;
        self.dirty = false;
        Ok(())
    }

    pub fn save(&mut self) -> Result<(), Error> {
        self.save_to_file(&self.fileinfo)?;
        self.dirty = false;
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub const fn is_file_loaded(&self) -> bool {
        self.fileinfo.has_path()
    }

    pub fn height(&self) -> LineIndex {
        self.lines.len()
    }

    pub fn insert_char(&mut self, character: char, at: Location) {
        debug_assert!(at.line_index <= self.height());

        if at.line_index == self.height() {
            self.lines.push(Line::from(&character.to_string()));
            self.dirty = true;
        } else if let Some(line) = self.lines.get_mut(at.line_index) {
            line.insert_char(character, at.grapheme_index);
            self.dirty = true;
        }
    }

    pub fn delete(&mut self, at: Location) {
        if let Some(line) = self.lines.get(at.line_index) {
            if at.grapheme_index >= line.grapheme_count()
                && self.height() > at.line_index.saturating_add(1)
            {
                let next_line = self.lines.remove(at.line_index.saturating_add(1));

                #[allow(clippy::indexing_slicing)]
                self.lines[at.line_index].append(&next_line);
                self.dirty = true;
            } else if at.grapheme_index < line.grapheme_count() {
                #[allow(clippy::indexing_slicing)]
                self.lines[at.line_index].delete(at.grapheme_index);
                self.dirty = true;
            }
        }
    }

    pub fn insert_newline(&mut self, at: Location) {
        if at.line_index == self.height() {
            self.lines.push(Line::default());
            self.dirty = true;
        } else if let Some(line) = self.lines.get_mut(at.line_index) {
            let new = line.split(at.grapheme_index);
            self.lines.insert(at.line_index.saturating_add(1), new);
            self.dirty = true;
        }
    }
}
