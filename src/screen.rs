mod overview;
mod update;

pub use overview::Overview;
pub use update::Update;

#[derive(Debug)]
pub enum Screen {
    Overview(Overview),
    Update(Update),
}
