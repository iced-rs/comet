use crate::beacon;
use crate::chart;
use crate::core::time::{Duration, SystemTime};

use std::collections::VecDeque;
use std::ops::{Add, RangeInclusive, Sub};

#[derive(Debug, Clone, Default)]
pub struct Timeline {
    events: VecDeque<beacon::Event>,
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
        Index(0)..=self.end()
    }

    pub fn end(&self) -> Index {
        Index(self.events.len())
    }

    pub fn index(&self, playhead: Playhead) -> Index {
        match playhead {
            Playhead::Live => Index(self.events.len()),
            Playhead::Paused(index) => index,
        }
    }

    pub fn push(&mut self, event: beacon::Event) {
        self.events.push_back(event);

        if self.events.len() > Self::MAX_SIZE {
            self.events.pop_front();
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
        let index = self.index(playhead.into());

        self.events
            .iter()
            .rev()
            .skip(self.events.len().saturating_sub(index.0))
    }

    pub fn seek_with_index(
        &self,
        playhead: impl Into<Playhead>,
    ) -> impl DoubleEndedIterator<Item = (Index, &beacon::Event)>
    + ExactSizeIterator<Item = (Index, &beacon::Event)>
    + Clone
    + '_ {
        let playhead = playhead.into();
        let index = self.index(playhead);

        self.seek(playhead)
            .enumerate()
            .map(move |(i, event)| (index - i as u32, event))
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

impl Add<u32> for Index {
    type Output = Self;

    fn add(self, rhs: u32) -> Self::Output {
        Self(self.0 + rhs as usize)
    }
}

impl Sub<u32> for Index {
    type Output = Self;

    fn sub(self, rhs: u32) -> Self::Output {
        Self(self.0.saturating_sub(rhs as usize))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timeframe {
    pub index: Index,
    pub at: SystemTime,
    pub duration: Duration,
}
