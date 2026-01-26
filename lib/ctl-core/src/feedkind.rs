use atx_feed::FeedKind;

/// Book Top feed kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Top;

/// Trade feed kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Trade;

impl FeedKind for Top {}
impl FeedKind for Trade {}