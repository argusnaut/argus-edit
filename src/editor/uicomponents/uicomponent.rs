use crate::prelude::*;

use std::io::Error;

pub trait UIComponent {
    fn set_needs_redraw(&mut self, value: bool);
    fn needs_redraw(&self) -> bool;
    fn set_size(&mut self, size: Size);
    fn draw(&mut self, origin_row: RowIndex) -> Result<(), Error>;

    fn resize(&mut self, size: Size) {
        self.set_size(size);
        self.set_needs_redraw(true);
    }

    fn render(&mut self, origin_row: RowIndex) {
        if self.needs_redraw() {
            if let Err(err) = self.draw(origin_row) {
                #[cfg(debug_assertions)]
                {
                    panic!("Could not render component: {err:?}");
                }
                #[cfg(not(debug_assertions))]
                {
                    let _ = err;
                }
            } else {
                self.set_needs_redraw(false);
            }
        }
    }
}
