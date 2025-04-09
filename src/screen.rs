mod overview;
mod present;
mod update;

pub use overview::Overview;
pub use present::Present;
pub use update::Update;

use crate::beacon::Event;

#[derive(Debug)]
pub enum Screen {
    Overview(Overview),
    Update(Update),
    Present(Present),
}

impl Screen {
    pub fn invalidate(&mut self) {
        match self {
            Screen::Overview(overview) => {
                overview.invalidate();
            }
            Screen::Update(update) => {
                update.invalidate();
            }
            Screen::Present(render) => {
                render.invalidate();
            }
        }
    }

    pub fn invalidate_by(&mut self, event: &Event) {
        match self {
            Screen::Overview(overview) => {
                overview.invalidate_by(event);
            }
            Screen::Update(update) => {
                update.invalidate_by(event);
            }
            Self::Present(render) => {
                render.invalidate_by(event);
            }
        }
    }
}
