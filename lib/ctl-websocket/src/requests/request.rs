use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::WSRequestId;

// ----------------------------- Websocket Request ------------------------------

/// A websocket request structure for Binance WebSocket APIs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WSRequest {
    /// The method and its associated parameters.
    #[serde(flatten)]
    pub kind: WSRequestKind,
    /// An optional identifier for the request.
    pub id: Option<WSRequestId>,
}

impl From<(WSRequestKind, Option<WSRequestId>)> for WSRequest {
    fn from((kind, id): (WSRequestKind, Option<WSRequestId>)) -> Self {
        WSRequest { kind, id }
    }
}

// -------------------------- Websocket Request Method & Parameters ---------------------------

/// A websocket request kind that ties method names to their 
/// corresponding parameter types for websocket requests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "method", content = "params")]
pub enum WSRequestKind {
    // ============================================
    // Websocket Market Data Streams
    // ============================================
    /// Subscribe to streams.
    /// https://github.com/binance/binance-spot-api-docs/blob/master/web-socket-streams.md#subscribe-to-a-stream
    #[serde(rename = "SUBSCRIBE")]
    Subscribe(Vec<String>),

    /// Unsubscribe from streams.
    /// https://github.com/binance/binance-spot-api-docs/blob/master/web-socket-streams.md#unsubscribe-to-a-stream
    #[serde(rename = "UNSUBSCRIBE")]
    Unsubscribe(Vec<String>),

    /// List current subscriptions.
    /// https://github.com/binance/binance-spot-api-docs/blob/master/web-socket-streams.md#listing-subscriptions
    #[serde(rename = "LIST_SUBSCRIPTIONS")]
    ListSubscriptions,

    /// Set a property.
    /// https://github.com/binance/binance-spot-api-docs/blob/master/web-socket-streams.md#setting-properties
    #[serde(rename = "SET_PROPERTY")]
    SetProperty(Vec<Value>),

    /// Get a property.
    /// https://github.com/binance/binance-spot-api-docs/blob/master/web-socket-streams.md#getting-properties
    #[serde(rename = "GET_PROPERTY")]
    GetProperty(Vec<String>),
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ======================== Serialization Tests ========================

    #[test]
    fn test_serialize_subscribe_with_int_id() {
        let req = WSRequest {
            kind: WSRequestKind::Subscribe(vec![
                "btcusdt@aggTrade".to_string(),
                "btcusdt@depth".to_string(),
            ]),
            id: Some(WSRequestId::Int(1)),
        };

        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(
            json,
            json!({
                "method": "SUBSCRIBE",
                "params": ["btcusdt@aggTrade", "btcusdt@depth"],
                "id": 1
            })
        );
    }

    #[test]
    fn test_serialize_subscribe_with_string_id() {
        let req = WSRequest {
            kind: WSRequestKind::Subscribe(vec!["btcusdt@trade".to_string()]),
            id: Some(WSRequestId::try_from("my-request-id").unwrap()),
        };

        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(
            json,
            json!({
                "method": "SUBSCRIBE",
                "params": ["btcusdt@trade"],
                "id": "my-request-id"
            })
        );
    }

    #[test]
    fn test_serialize_subscribe_without_id() {
        let req = WSRequest {
            kind: WSRequestKind::Subscribe(vec!["btcusdt@kline_1m".to_string()]),
            id: None,
        };

        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(
            json,
            json!({
                "method": "SUBSCRIBE",
                "params": ["btcusdt@kline_1m"],
                "id": null
            })
        );
    }

    #[test]
    fn test_serialize_unsubscribe() {
        let req = WSRequest {
            kind: WSRequestKind::Unsubscribe(vec!["btcusdt@depth".to_string()]),
            id: Some(WSRequestId::Int(312)),
        };

        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(
            json,
            json!({
                "method": "UNSUBSCRIBE",
                "params": ["btcusdt@depth"],
                "id": 312
            })
        );
    }

    #[test]
    fn test_serialize_list_subscriptions() {
        let req = WSRequest {
            kind: WSRequestKind::ListSubscriptions,
            id: Some(WSRequestId::Int(3)),
        };

        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(
            json,
            json!({
                "method": "LIST_SUBSCRIPTIONS",
                "id": 3
            })
        );
    }

    #[test]
    fn test_serialize_set_property() {
        let req = WSRequest {
            kind: WSRequestKind::SetProperty(vec![json!("combined"), json!(true)]),
            id: Some(WSRequestId::Int(5)),
        };

        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(
            json,
            json!({
                "method": "SET_PROPERTY",
                "params": ["combined", true],
                "id": 5
            })
        );
    }

    #[test]
    fn test_serialize_get_property() {
        let req = WSRequest {
            kind: WSRequestKind::GetProperty(vec!["combined".to_string()]),
            id: Some(WSRequestId::Int(2)),
        };

        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(
            json,
            json!({
                "method": "GET_PROPERTY",
                "params": ["combined"],
                "id": 2
            })
        );
    }

    // ======================== Deserialization Tests ========================

    #[test]
    fn test_deserialize_subscribe_with_int_id() {
        let json = r#"{"method":"SUBSCRIBE","params":["btcusdt@aggTrade","btcusdt@depth"],"id":1}"#;
        let req: WSRequest = serde_json::from_str(json).unwrap();

        assert_eq!(
            req.kind,
            WSRequestKind::Subscribe(vec![
                "btcusdt@aggTrade".to_string(),
                "btcusdt@depth".to_string()
            ])
        );
        assert_eq!(req.id, Some(WSRequestId::Int(1)));
    }

    #[test]
    fn test_deserialize_subscribe_with_string_id() {
        let json = r#"{"method":"SUBSCRIBE","params":["btcusdt@trade"],"id":"abc123"}"#;
        let req: WSRequest = serde_json::from_str(json).unwrap();

        assert_eq!(
            req.kind,
            WSRequestKind::Subscribe(vec!["btcusdt@trade".to_string()])
        );
        assert_eq!(
            req.id,
            Some(WSRequestId::try_from("abc123").unwrap())
        );
    }

    #[test]
    fn test_deserialize_subscribe_with_null_id() {
        let json = r#"{"method":"SUBSCRIBE","params":["btcusdt@kline_1m"],"id":null}"#;
        let req: WSRequest = serde_json::from_str(json).unwrap();

        assert_eq!(
            req.kind,
            WSRequestKind::Subscribe(vec!["btcusdt@kline_1m".to_string()])
        );
        assert_eq!(req.id, None);
    }

    #[test]
    fn test_deserialize_unsubscribe() {
        let json = r#"{"method":"UNSUBSCRIBE","params":["btcusdt@depth"],"id":312}"#;
        let req: WSRequest = serde_json::from_str(json).unwrap();

        assert_eq!(
            req.kind,
            WSRequestKind::Unsubscribe(vec!["btcusdt@depth".to_string()])
        );
        assert_eq!(req.id, Some(WSRequestId::Int(312)));
    }

    #[test]
    fn test_deserialize_list_subscriptions() {
        let json = r#"{"method":"LIST_SUBSCRIPTIONS","id":3}"#;
        let req: WSRequest = serde_json::from_str(json).unwrap();

        assert_eq!(req.kind, WSRequestKind::ListSubscriptions);
        assert_eq!(req.id, Some(WSRequestId::Int(3)));
    }

    #[test]
    fn test_deserialize_set_property() {
        let json = r#"{"method":"SET_PROPERTY","params":["combined",true],"id":5}"#;
        let req: WSRequest = serde_json::from_str(json).unwrap();

        assert_eq!(
            req.kind,
            WSRequestKind::SetProperty(vec![json!("combined"), json!(true)])
        );
        assert_eq!(req.id, Some(WSRequestId::Int(5)));
    }

    #[test]
    fn test_deserialize_get_property() {
        let json = r#"{"method":"GET_PROPERTY","params":["combined"],"id":2}"#;
        let req: WSRequest = serde_json::from_str(json).unwrap();

        assert_eq!(
            req.kind,
            WSRequestKind::GetProperty(vec!["combined".to_string()])
        );
        assert_eq!(req.id, Some(WSRequestId::Int(2)));
    }

    // ======================== Round-trip Tests ========================

    #[test]
    fn test_roundtrip_subscribe() {
        let original = WSRequest {
            kind: WSRequestKind::Subscribe(vec![
                "btcusdt@aggTrade".to_string(),
                "ethusdt@depth".to_string(),
            ]),
            id: Some(WSRequestId::Int(42)),
        };

        let json_str = serde_json::to_string(&original).unwrap();
        let deserialized: WSRequest = serde_json::from_str(&json_str).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_roundtrip_set_property_with_mixed_types() {
        let original = WSRequest {
            kind: WSRequestKind::SetProperty(vec![json!("combined"), json!(false)]),
            id: Some(WSRequestId::try_from("uuid-1234-5678").unwrap()),
        };

        let json_str = serde_json::to_string(&original).unwrap();
        let deserialized: WSRequest = serde_json::from_str(&json_str).unwrap();

        assert_eq!(original, deserialized);
    }

    // ======================== WSRequestId Tests ========================

    #[test]
    fn test_request_id_from_i64() {
        let id: WSRequestId = 123i64.into();
        assert_eq!(id, WSRequestId::Int(123));
    }

    #[test]
    fn test_request_id_from_u64() {
        let id: WSRequestId = 456u64.into();
        assert_eq!(id, WSRequestId::Int(456));
    }

    #[test]
    fn test_request_id_try_from_str_valid() {
        let id = WSRequestId::try_from("valid-id").unwrap();
        assert!(matches!(id, WSRequestId::String(_)));
    }

    #[test]
    fn test_request_id_try_from_str_max_length() {
        // Exactly 36 characters should work
        let id_str = "a".repeat(36);
        let result = WSRequestId::try_from(id_str.as_str());
        assert!(result.is_ok());
    }

    #[test]
    fn test_request_id_try_from_str_too_long() {
        // 37 characters should fail
        let id_str = "a".repeat(37);
        let result = WSRequestId::try_from(id_str.as_str());
        assert!(result.is_err());
    }

    // ======================== From Tuple Tests ========================

    #[test]
    fn test_ws_request_from_tuple() {
        let kind = WSRequestKind::Subscribe(vec!["btcusdt@trade".to_string()]);
        let id = Some(WSRequestId::Int(1));

        let req: WSRequest = (kind.clone(), id.clone()).into();

        assert_eq!(req.kind, kind);
        assert_eq!(req.id, id);
    }
}