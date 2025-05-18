use crate::prelude::*;

use std::{
    cmp::min,
    fmt::{self, Display, Formatter},
    ops::{Deref, Range},
};

mod graphemewidth;
mod textfragment;

use graphemewidth::GraphemeWidth;
use textfragment::TextFragment;

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use super::annotatedstring::{AnnotatedString, AnnotationType};

#[derive(Default, Clone)]
pub struct Line {
    fragments: Vec<TextFragment>,
    string: String,
}

impl Line {
    pub fn from(line_str: &str) -> Self {
        debug_assert!(line_str.is_empty() || line_str.lines().count() == 1);
        let fragments = Self::str_to_fragments(line_str);
        Self {
            fragments,
            string: String::from(line_str),
        }
    }

    fn str_to_fragments(line_str: &str) -> Vec<TextFragment> {
        line_str
            .grapheme_indices(true)
            .map(|(byte_index, grapheme)| {
                let (replacement, rendered_width) = Self::get_replacement_character(grapheme)
                    .map_or_else(
                        || {
                            let unicode_width = grapheme.width();
                            let rendered_width = match unicode_width {
                                0 | 1 => GraphemeWidth::Half,
                                _ => GraphemeWidth::Full,
                            };
                            (None, rendered_width)
                        },
                        |replacement| (Some(replacement), GraphemeWidth::Half),
                    );

                TextFragment {
                    grapheme: grapheme.to_string(),
                    rendered_width,
                    replacement,
                    start: byte_index,
                }
            })
            .collect()
    }

    fn rebuild_fragments(&mut self) {
        self.fragments = Self::str_to_fragments(&self.string);
    }

    fn get_replacement_character(for_str: &str) -> Option<char> {
        let width = for_str.width();

        match for_str {
            " " => None,
            "\t" => Some(' '),
            _ if width > 0 && for_str.trim().is_empty() => Some('␣'),
            _ if width == 0 => {
                let mut chars = for_str.chars();
                if let Some(ch) = chars.next() {
                    if ch.is_control() && chars.next().is_none() {
                        return Some('▯');
                    }
                }
                Some('·')
            }
            _ => None,
        }
    }

    pub fn get_visible_graphemes(&self, range: Range<GraphemeIndex>) -> String {
        self.get_annotated_visible_substr(range, None, None)
            .to_string()
    }

    pub fn get_annotated_visible_substr(
        &self,
        range: Range<ColIndex>,
        query: Option<&str>,
        selected_match: Option<GraphemeIndex>,
    ) -> AnnotatedString {
        if range.start > range.end {
            return AnnotatedString::default();
        }

        let mut result = AnnotatedString::from(&self.string);

        if let Some(query) = query {
            if !query.is_empty() {
                self.find_all(query, 0..self.string.len()).iter().for_each(
                    |(start, grapheme_index)| {
                        if let Some(selected_match) = selected_match {
                            if *grapheme_index == selected_match {
                                result.add_annotation(
                                    AnnotationType::SelectedMatch,
                                    *start,
                                    start.saturating_add(query.len()),
                                );
                                return;
                            }
                        }
                        result.add_annotation(
                            AnnotationType::Match,
                            *start,
                            start.saturating_add(query.len()),
                        );
                    },
                );
            }
        }

        let mut fragment_start = self.width();

        for fragment in self.fragments.iter().rev() {
            let fragment_end = fragment_start;
            fragment_start = fragment_start.saturating_sub(fragment.rendered_width.into());

            if fragment_end >= range.end {
                continue;
            }

            if fragment_start < range.end && fragment_end > range.end {
                result.replace(fragment.start, self.string.len(), "⋯");
                continue;
            } else if fragment_start == range.end {
                result.truncate_right_from(fragment.start);
                continue;
            }

            if fragment_end <= range.start {
                result.truncate_left_until(
                    fragment
                        .start
                        .saturating_add(fragment.grapheme.len()),
                );
                break;
            } else if fragment_start < range.start && fragment_end > range.start {
                result.replace(
                    0,
                    fragment
                        .start
                        .saturating_add(fragment.grapheme.len()),
                    "⋯",
                );
                break;
            }
            if fragment_start >= range.start && fragment_end <= range.end {
                if let Some(replacement) = fragment.replacement {
                    let start = fragment.start;
                    let end = start.saturating_add(fragment.grapheme.len());
                    result.replace(start, end, &replacement.to_string());
                }
            }
        }
        result
    }

    pub fn grapheme_count(&self) -> GraphemeIndex {
        self.fragments.len()
    }

    pub fn width_until(&self, grapheme_index: GraphemeIndex) -> ColIndex {
        self.fragments
            .iter()
            .take(grapheme_index)
            .map(|fragment| match fragment.rendered_width {
                GraphemeWidth::Half => 1,
                GraphemeWidth::Full => 2,
            })
            .sum()
    }

    pub fn width(&self) -> ColIndex {
        self.width_until(self.grapheme_count())
    }

    pub fn insert_char(&mut self, character: char, at: GraphemeIndex) {
        debug_assert!(at.saturating_sub(1) <= self.grapheme_count());
        if let Some(fragment) = self.fragments.get(at) {
            self.string.insert(fragment.start, character);
        } else {
            self.string.push(character);
        }

        self.rebuild_fragments();
    }

    pub fn append_char(&mut self, character: char) {
        self.insert_char(character, self.grapheme_count());
    }

    pub fn delete(&mut self, at: GraphemeIndex) {
        debug_assert!(at <= self.grapheme_count());
        if let Some(fragment) = self.fragments.get(at) {
            let start = fragment.start;
            let end = fragment
                .start
                .saturating_add(fragment.grapheme.len());

            self.string.drain(start..end);
            self.rebuild_fragments();
        }
    }

    pub fn delete_last(&mut self) {
        self.delete(self.grapheme_count().saturating_sub(1));
    }

    pub fn append(&mut self, other: &Self) {
        self.string.push_str(&other.string);
        self.rebuild_fragments();
    }

    pub fn split(&mut self, at: GraphemeIndex) -> Self {
        if let Some(fragment) = self.fragments.get(at) {
            let remainder = self.string.split_off(fragment.start);
            self.rebuild_fragments();
            Self::from(&remainder)
        } else {
            Self::default()
        }
    }

    pub fn byte_index_to_grapheme_index(&self, byte_index: ByteIndex) -> Option<GraphemeIndex> {
        if byte_index > self.string.len() {
            return None;
        }

        self.fragments
            .iter()
            .position(|fragment| fragment.start >= byte_index)
    }

    fn grapheme_index_to_byte_index(&self, grapheme_index: GraphemeIndex) -> ByteIndex {
        debug_assert!(grapheme_index <= self.grapheme_count());

        if grapheme_index == 0 || self.grapheme_count() == 0 {
            return 0;
        }

        self.fragments.get(grapheme_index).map_or_else(
            || {
                #[cfg(debug_assertions)]
                {
                    panic!("Fragment not found for grapheme index: {grapheme_index:?}");
                }

                #[cfg(not(debug_assertions))]
                {
                    0
                }
            },
            |fragment| fragment.start,
        )
    }

    pub fn search_forward(
        &self,
        query: &str,
        from_grapheme_index: GraphemeIndex,
    ) -> Option<GraphemeIndex> {
        debug_assert!(from_grapheme_index <= self.grapheme_count());

        if from_grapheme_index == self.grapheme_count() {
            return None;
        }

        let start = self.grapheme_index_to_byte_index(from_grapheme_index);

        self.find_all(query, start..self.string.len())
            .first()
            .map(|(_, grapheme_index)| *grapheme_index)
    }

    pub fn search_backward(
        &self,
        query: &str,
        from_grapheme_index: GraphemeIndex,
    ) -> Option<GraphemeIndex> {
        debug_assert!(from_grapheme_index <= self.grapheme_count());

        if from_grapheme_index == 0 {
            return None;
        }

        let end_byte_index = if from_grapheme_index == self.grapheme_count() {
            self.string.len()
        } else {
            self.grapheme_index_to_byte_index(from_grapheme_index)
        };

        self.find_all(query, 0..end_byte_index)
            .last()
            .map(|(_, grapheme_index)| *grapheme_index)
    }

    fn find_all(&self, query: &str, range: Range<ByteIndex>) -> Vec<(ByteIndex, GraphemeIndex)> {
        let end = min(range.end, self.string.len());
        let start = range.start;

        debug_assert!(start <= end);
        debug_assert!(start <= self.string.len());

        self.string.get(start..end).map_or_else(Vec::new, |substr| {
            let potential_matches: Vec<ByteIndex> = substr
                .match_indices(query) // find _potential_ matches within the substring
                .map(|(relative_start_index, _)| {
                    relative_start_index.saturating_add(start) //convert their relative indices to absolute indices
                })
                .collect();

            self.match_grapheme_clusters(&potential_matches, query)
        })
    }

    fn match_grapheme_clusters(
        &self,
        matches: &[ByteIndex],
        query: &str,
    ) -> Vec<(ByteIndex, GraphemeIndex)> {
        let grapheme_count = query.graphemes(true).count();

        matches
            .iter()
            .filter_map(|&start| {
                self.byte_index_to_grapheme_index(start)
                    .and_then(|grapheme_index| {
                        self.fragments
                            .get(grapheme_index..grapheme_index.saturating_add(grapheme_count)) // get all fragments that should be part of the match
                            .and_then(|fragments| {
                                let substring = fragments
                                    .iter()
                                    .map(|fragment| fragment.grapheme.as_str())
                                    .collect::<String>();

                                (substring == query).then_some((start, grapheme_index))
                            })
                    })
            })
            .collect()
    }
}

impl Display for Line {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.string)
    }
}

impl Deref for Line {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.string
    }
}
