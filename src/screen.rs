mod overview;
mod present;
mod update;

pub mod custom;

pub use custom::Custom;
pub use overview::Overview;
pub use present::Present;
pub use update::Update;

use crate::beacon::Event;

#[derive(Debug)]
pub enum Screen {
    Overview(Overview),
    Update(Update),
    Present(Present),
    Custom(Custom),
}

impl Screen {
    pub fn invalidate(&mut self) {
        match self {
            Self::Overview(overview) => {
                overview.invalidate();
            }
            Self::Update(update) => {
                update.invalidate();
            }
            Self::Present(render) => {
                render.invalidate();
            }
            Self::Custom(custom) => {
                custom.invalidate();
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
            Self::Custom(custom) => {
                custom.invalidate_by(event);
            }
        }
    }
}
