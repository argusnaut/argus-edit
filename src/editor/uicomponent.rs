use super::Size;
use std::io::Error;

pub trait UIComponent {
    fn set_needs_redraw(&mut self, value: bool);
    fn needs_redraw(&self) -> bool;
    fn set_size(&mut self, size: Size);
    fn draw(&mut self, origin_row: usize) -> Result<(), Error>;

    fn resize(&mut self, size: Size) {
        self.set_size(size);
        self.set_needs_redraw(true);
    }

    fn render(&mut self, origin_row: usize) {
        if self.needs_redraw() {
            match self.draw(origin_row) {
                Ok(()) => self.set_needs_redraw(false),
                Err(err) => {
                    #[cfg(debug_assertions)]
                    {
                        panic!("Could not render component: {err:?}");
                    }
                }
            }
        }
    }
}
