use serde_json::Value as JsonValue;
use svc_agent::mqtt::{compat::IntoEnvelope, IncomingRequest, Publish, ResponseStatus};
use svc_error::Error as SvcError;

////////////////////////////////////////////////////////////////////////////////

pub(crate) type EchoRequest = IncomingRequest<JsonValue>;

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub(crate) struct State {}

impl State {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl State {
    pub(crate) async fn echo(&self, inreq: EchoRequest) -> Result<impl Publish, SvcError> {
        let resp = inreq.to_response(inreq.payload(), ResponseStatus::OK);
        resp.into_envelope().map_err(Into::into)
    }
}
