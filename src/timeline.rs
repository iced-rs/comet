use crate::beacon;
use crate::beacon::span;
use crate::chart;
use crate::core::time::{Duration, SystemTime};

use std::collections::VecDeque;
use std::ops::{Add, RangeInclusive, Sub};

#[derive(Debug, Clone, Default)]
pub struct Timeline {
    events: VecDeque<beacon::Event>,
    updates: VecDeque<Update>,
    update_rate: VecDeque<Bucket>,
    removed: usize,
}

impl Timeline {
    // TODO: Make configurable
    const MAX_SIZE: usize = 1_000_000;

    pub fn new() -> Self {
        Self::default()
    }

    pub fn capacity(&self) -> usize {
        Self::MAX_SIZE
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn range(&self) -> RangeInclusive<Index> {
        Index(self.removed)..=self.end()
    }

    pub fn end(&self) -> Index {
        Index(self.events.len() + self.removed)
    }

    pub fn index(&self, playhead: Playhead) -> Index {
        match playhead {
            Playhead::Live => self.end(),
            Playhead::Paused(index) => index,
        }
    }

    pub fn push(&mut self, event: beacon::Event) {
        if let beacon::Event::SpanFinished {
            span:
                span::Span::Update {
                    number,
                    tasks,
                    subscriptions,
                    ref message,
                    ..
                },
            at,
            duration,
            ..
        } = event
        {
            self.updates.push_back(Update {
                index: self.end(),
                message: message.clone(),
                duration,
                number,
                tasks,
                subscriptions,
            });

            let second = at
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            match self.update_rate.back_mut() {
                Some(update_rate) if update_rate.second == second => {
                    update_rate.at = at;
                    update_rate.total += 1;
                }
                _ => {
                    self.update_rate.push_back(Bucket {
                        index: self.end(),
                        at,
                        second,
                        total: 1,
                    });
                }
            }
        }

        self.events.push_back(event);

        if self.events.len() > Self::MAX_SIZE {
            if let Some(beacon::Event::SpanFinished {
                span: span::Span::Update { .. },
                at,
                ..
            }) = self.events.pop_front()
            {
                self.updates.pop_front();

                if self
                    .update_rate
                    .front()
                    .is_some_and(|bucket| bucket.at < at)
                {
                    self.update_rate.pop_front();
                }
            }

            self.removed += 1;
        }
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }

    pub fn seek(
        &self,
        playhead: impl Into<Playhead>,
    ) -> impl DoubleEndedIterator<Item = &beacon::Event>
    + ExactSizeIterator<Item = &beacon::Event>
    + Clone
    + '_ {
        let index = self.index(playhead.into()) - self.removed;

        self.events.range(0..index.0).rev()
    }

    pub fn seek_with_index(
        &self,
        playhead: impl Into<Playhead>,
    ) -> impl DoubleEndedIterator<Item = (Index, &beacon::Event)>
    + ExactSizeIterator<Item = (Index, &beacon::Event)>
    + Clone
    + '_ {
        let playhead = playhead.into();
        let index = self.index(playhead) - self.removed;

        self.seek(playhead)
            .enumerate()
            .map(move |(i, event)| (index - i, event))
    }

    pub fn timeframes(
        &self,
        playhead: Playhead,
        stage: chart::Stage,
    ) -> impl DoubleEndedIterator<Item = Timeframe> + Clone + '_ {
        self.seek_with_index(playhead)
            .filter_map(move |(index, event)| match event {
                beacon::Event::SpanFinished { at, duration, span }
                    if chart::Stage::from(span.stage()) == stage =>
                {
                    Some(Timeframe {
                        index,
                        at: *at,
                        duration: *duration,
                    })
                }
                _ => None,
            })
    }

    pub fn updates(
        &self,
        playhead: impl Into<Playhead>,
    ) -> impl DoubleEndedIterator<Item = Update> + Clone + '_ {
        let index = self.index(playhead.into());

        let start = match self
            .updates
            .binary_search_by(|update| update.index.cmp(&index))
        {
            Ok(i) | Err(i) => i,
        };

        self.updates.range(0..start).cloned().rev()
    }

    pub fn update_rate(
        &self,
        playhead: impl Into<Playhead>,
    ) -> impl DoubleEndedIterator<Item = Bucket> + Clone + '_ {
        let index = self.index(playhead.into());

        let start = match self
            .update_rate
            .binary_search_by(|update| update.index.cmp(&index))
        {
            Ok(i) | Err(i) => i,
        };

        self.update_rate.range(0..start).cloned().rev()
    }

    pub fn time_at(&self, playhead: Playhead) -> Option<SystemTime> {
        self.seek(playhead).next().map(beacon::Event::at)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Playhead {
    Live,
    Paused(Index),
}

impl Playhead {
    pub fn is_live(self) -> bool {
        matches!(self, Self::Live)
    }
}

impl From<Index> for Playhead {
    fn from(index: Index) -> Self {
        Self::Paused(index)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Index(usize);

impl From<u8> for Index {
    fn from(n: u8) -> Self {
        Self(usize::from(n))
    }
}

impl From<Index> for f64 {
    fn from(index: Index) -> Self {
        index.0 as Self
    }
}

impl num_traits::FromPrimitive for Index {
    fn from_i64(n: i64) -> Option<Self> {
        if n < 0 { None } else { Some(Self(n as usize)) }
    }

    fn from_u64(n: u64) -> Option<Self> {
        Some(Self(n as usize))
    }
}

impl Add<usize> for Index {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl Sub<usize> for Index {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self::Output {
        Self(self.0.saturating_sub(rhs))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timeframe {
    pub index: Index,
    pub at: SystemTime,
    pub duration: Duration,
}

#[derive(Debug, Clone)]
pub struct Update {
    pub index: Index,
    pub duration: Duration,
    pub number: usize,
    pub tasks: usize,
    pub subscriptions: usize,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct Bucket {
    pub index: Index,
    pub at: SystemTime,
    pub second: u64,
    pub total: usize,
}
