use crate::proto::Flow;

/// Request flow constant - process incoming HTTP requests.
pub const FLOW_REQUEST: Flow = Flow::Request;

/// Response flow constant - process outgoing HTTP responses.
pub const FLOW_RESPONSE: Flow = Flow::Response;
