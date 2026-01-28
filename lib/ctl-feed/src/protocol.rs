
impl FeedProtocol<Top> for BSWebsocketConn {
    fn update(&mut self, streams: &Streams<Top>) -> Result<(), Self::FeedProtocolError> {
        // For BookTicker (Top), the stream name format is: <symbol>@bookTicker
        // The stream name in Streams<Top> is expected to be the symbol (lowercase)
        // e.g., "btcusdt" -> "btcusdt@bookTicker"
        let stream_names: Vec<String> = streams
            .iter()
            .map(|s| format!("{}@bookTicker", s.name))
            .collect();

        let stream_refs: Vec<&str> = stream_names.iter().map(|s| s.as_str()).collect();

        // For simplicity, we subscribe to all streams.
        // A more sophisticated implementation would track current subscriptions
        // and only subscribe/unsubscribe the diff.
        self.subscribe(&stream_refs)
    }
}

impl FeedProtocol<Trade> for BSWebsocketConn {
    fn update(&mut self, streams: &Streams<Trade>) -> Result<(), Self::FeedProtocolError> {
        // For Trade, the stream name format is: <symbol>@trade
        // The stream name in Streams<Trade> is expected to be the symbol (lowercase)
        // e.g., "btcusdt" -> "btcusdt@trade"
        let stream_names: Vec<String> = streams
            .iter()
            .map(|s| format!("{}@trade", s.name))
            .collect();

        let stream_refs: Vec<&str> = stream_names.iter().map(|s| s.as_str()).collect();

        // For simplicity, we subscribe to all streams.
        // A more sophisticated implementation would track current subscriptions
        // and only subscribe/unsubscribe the diff.
        self.subscribe(&stream_refs)
    }
}
