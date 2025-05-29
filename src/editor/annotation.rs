use super::AnnotationType;
use crate::prelude::ByteIndex;

#[derive(Copy, Clone, Debug)]
#[allow(clippy::struct_field_names)]
pub struct Annotation {
    pub annotation_type: AnnotationType,
    pub start: ByteIndex,
    pub end: ByteIndex,
}

impl Annotation {
    pub fn shift(&mut self, offset: ByteIndex) {
        self.start = self.start.saturating_add(offset);
        self.end = self.end.saturating_add(offset);
    }
}
