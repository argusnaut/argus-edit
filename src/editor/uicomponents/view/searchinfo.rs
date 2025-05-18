use crate::editor::Line;
use crate::prelude::*;

pub struct SearchInfo {
    pub prev_location: Location,
    pub prev_location_offset: Position,
    pub query: Option<Line>,
}
