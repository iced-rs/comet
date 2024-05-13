use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::tree::{self, Tree};
use iced::advanced::{self, Clipboard, Shell, Widget};
use iced::alignment;
use iced::event::{self, Event};
use iced::mouse;
use iced::time::{Duration, Instant};
use iced::widget::text;
use iced::window;
use iced::{Color, Element, Length, Pixels, Rectangle, Size};

#[derive(Debug)]
pub struct AnimatedText<'a, Theme, Renderer>
where
    Theme: text::Catalog,
    Renderer: advanced::text::Renderer,
{
    fragment: text::Fragment<'a>,
    size: Option<Pixels>,
    line_height: text::LineHeight,
    width: Length,
    height: Length,
    horizontal_alignment: alignment::Horizontal,
    vertical_alignment: alignment::Vertical,
    font: Option<Renderer::Font>,
    shaping: text::Shaping,
    class: Theme::Class<'a>,
    duration: Duration,
}

impl<'a, Theme, Renderer> AnimatedText<'a, Theme, Renderer>
where
    Theme: text::Catalog,
    Renderer: advanced::text::Renderer,
{
    const TICK_RATE_MILLIS: u64 = 25;

    pub fn new(fragment: impl text::IntoFragment<'a>) -> Self {
        Self {
            fragment: fragment.into_fragment(),
            size: None,
            line_height: text::LineHeight::default(),
            font: None,
            width: Length::Shrink,
            height: Length::Shrink,
            horizontal_alignment: alignment::Horizontal::Left,
            vertical_alignment: alignment::Vertical::Top,
            shaping: text::Shaping::Basic,
            class: Theme::default(),
            duration: Duration::from_millis(500),
        }
    }

    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = Some(size.into());
        self
    }

    pub fn line_height(mut self, line_height: impl Into<text::LineHeight>) -> Self {
        self.line_height = line_height.into();
        self
    }

    pub fn font(mut self, font: impl Into<Renderer::Font>) -> Self {
        self.font = Some(font.into());
        self
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    pub fn horizontal_alignment(mut self, alignment: alignment::Horizontal) -> Self {
        self.horizontal_alignment = alignment;
        self
    }

    pub fn vertical_alignment(mut self, alignment: alignment::Vertical) -> Self {
        self.vertical_alignment = alignment;
        self
    }

    pub fn shaping(mut self, shaping: text::Shaping) -> Self {
        self.shaping = shaping;
        self
    }
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme) -> text::Style + 'a) -> Self
    where
        Theme::Class<'a>: From<text::StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as text::StyleFn<'a, Theme>).into();
        self
    }
    pub fn color(self, color: impl Into<Color>) -> Self
    where
        Theme::Class<'a>: From<text::StyleFn<'a, Theme>>,
    {
        self.color_maybe(Some(color))
    }

    pub fn color_maybe(self, color: Option<impl Into<Color>>) -> Self
    where
        Theme::Class<'a>: From<text::StyleFn<'a, Theme>>,
    {
        let color = color.map(Into::into);

        self.style(move |_theme| text::Style { color })
    }

    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }
}

/// The internal state of a [`Text`] widget.
#[derive(Debug)]
pub struct State<P: advanced::text::Paragraph> {
    internal: text::State<P>,
    animation: Animation,
    last_fragment: String,
}

#[derive(Debug)]
enum Animation {
    Ticking {
        fragment: String,
        ticks: u64,
        next_redraw: Instant,
    },
    Done,
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for AnimatedText<'a, Theme, Renderer>
where
    Theme: text::Catalog,
    Renderer: advanced::text::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State<Renderer::Paragraph>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State {
            internal: text::State::<Renderer::Paragraph>::default(),
            animation: Animation::Ticking {
                fragment: String::new(),
                ticks: 0,
                next_redraw: Instant::now(),
            },
            last_fragment: String::new(),
        })
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = &mut tree.state.downcast_mut::<State<Renderer::Paragraph>>();

        if state.last_fragment != self.fragment {
            state.animation = Animation::Ticking {
                fragment: String::new(),
                ticks: 0,
                next_redraw: Instant::now(),
            };
            state.last_fragment = self.fragment.clone().into_owned();
        }

        let fragment = match &state.animation {
            Animation::Ticking { fragment, .. } => fragment,
            Animation::Done { .. } => self.fragment.as_ref(),
        };

        text::layout(
            &mut state.internal,
            renderer,
            limits,
            self.width,
            self.height,
            fragment,
            self.line_height,
            self.size,
            self.font,
            self.horizontal_alignment,
            self.vertical_alignment,
            self.shaping,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        defaults: &renderer::Style,
        layout: Layout<'_>,
        _cursor_position: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State<Renderer::Paragraph>>();
        let style = theme.style(&self.class);

        text::draw(renderer, defaults, layout, &state.internal, style, viewport);
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        use rand::Rng;

        match event {
            Event::Window(_, window::Event::RedrawRequested(now)) => {
                let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();

                match &mut state.animation {
                    Animation::Ticking {
                        fragment,
                        next_redraw,
                        ticks,
                    } => {
                        if *next_redraw <= now {
                            *ticks += 1;

                            let mut rng = rand::thread_rng();
                            let progress = (self.fragment.len() as f32
                                / self.duration.as_millis() as f32
                                * (*ticks * Self::TICK_RATE_MILLIS) as f32)
                                as usize;

                            if progress >= self.fragment.len() {
                                state.animation = Animation::Done;
                                shell.invalidate_layout();

                                return event::Status::Ignored;
                            }

                            *fragment = self
                                .fragment
                                .chars()
                                .take(progress as usize)
                                .chain(
                                    std::iter::from_fn(|| Some(rng.gen_range('!'..'z'))).take(
                                        self.fragment.len().saturating_sub(progress as usize),
                                    ),
                                )
                                .collect::<String>();

                            *next_redraw = now + Duration::from_millis(Self::TICK_RATE_MILLIS);

                            shell.invalidate_layout();
                        }

                        shell.request_redraw(window::RedrawRequest::At(*next_redraw));
                    }
                    Animation::Done { .. } => {}
                }
            }
            _ => {}
        }

        event::Status::Ignored
    }
}

impl<'a, Message, Theme, Renderer> From<AnimatedText<'a, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Theme: text::Catalog + 'a,
    Renderer: advanced::text::Renderer + 'a,
{
    fn from(text: AnimatedText<'a, Theme, Renderer>) -> Element<'a, Message, Theme, Renderer> {
        Element::new(text)
    }
}
