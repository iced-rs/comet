use crate::beacon::span;
use crate::Module;

use iced::widget::pane_grid;
use iced::window;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Board {
    Overview,
    Update,
}

impl Board {
    pub const ALL: &'static [Self] = &[Self::Overview, Self::Update];

    pub fn modules(self) -> pane_grid::Configuration<Module> {
        match self {
            Self::Overview => overview_modules(),
            Self::Update => update_modules(),
        }
    }
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Overview => "Overview",
            Self::Update => "Update",
        })
    }
}

fn overview_modules() -> pane_grid::Configuration<Module> {
    let update_and_view = vsplit(
        Module::performance_chart(span::Stage::Update),
        Module::performance_chart(span::Stage::View(window::Id::MAIN)),
    );

    let layout_and_interact = vsplit(
        Module::performance_chart(span::Stage::Layout(window::Id::MAIN)),
        Module::performance_chart(span::Stage::Interact(window::Id::MAIN)),
    );

    let draw_and_present = vsplit(
        Module::performance_chart(span::Stage::Draw(window::Id::MAIN)),
        Module::performance_chart(span::Stage::Present(window::Id::MAIN)),
    );

    pane_grid::Configuration::Split {
        axis: pane_grid::Axis::Horizontal,
        ratio: 1.0 / 3.0,
        a: Box::new(update_and_view),
        b: Box::new(pane_grid::Configuration::Split {
            axis: pane_grid::Axis::Horizontal,
            ratio: 0.5,
            a: Box::new(layout_and_interact),
            b: Box::new(draw_and_present),
        }),
    }
}

fn update_modules() -> pane_grid::Configuration<Module> {
    let update = pane_grid::Configuration::Pane(Module::performance_chart(span::Stage::Update));

    let commands_and_subscriptions =
        vsplit(Module::commands_spawned(), Module::subscriptions_alive());

    let message_rate_and_log = vsplit(Module::message_rate(), Module::message_log());

    pane_grid::Configuration::Split {
        axis: pane_grid::Axis::Horizontal,
        ratio: 1.0 / 3.0,
        a: Box::new(update),
        b: Box::new(pane_grid::Configuration::Split {
            axis: pane_grid::Axis::Horizontal,
            ratio: 0.5,
            a: Box::new(commands_and_subscriptions),
            b: Box::new(message_rate_and_log),
        }),
    }
}

fn vsplit(left: Module, right: Module) -> pane_grid::Configuration<Module> {
    pane_grid::Configuration::Split {
        axis: pane_grid::Axis::Vertical,
        ratio: 0.5,
        a: Box::new(pane_grid::Configuration::Pane(left)),
        b: Box::new(pane_grid::Configuration::Pane(right)),
    }
}
