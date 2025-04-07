mod overview;
mod update;

pub use overview::Overview;
pub use update::Update;

use crate::beacon::Event;

#[derive(Debug)]
pub enum Screen {
    Overview(Overview),
    Update(Update),
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
        }
    }
}
