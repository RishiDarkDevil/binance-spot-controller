use atx_feed::FeedGroup;
use ctl_core::Top;
use ctl_websocket::BSWebsocketConn;

pub enum BSFeedGroup {
    JsonTop(FeedGroup<BSWebsocketConn, Top, >)
}