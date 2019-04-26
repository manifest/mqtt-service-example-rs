use failure::{format_err, Error};
use futures::{executor::ThreadPool, task::SpawnExt, StreamExt};
use log::{error, info};
use std::sync::Arc;
use std::thread;
use svc_agent::mqtt::{compat, Agent, AgentBuilder, ConnectionMode, Notification, QoS};
use svc_agent::{AgentId, Authenticable, SharedGroup, Subscription};

struct State {
    message: endpoint::message::State,
}

pub(crate) async fn run() -> Result<(), Error> {
    // Config
    let config = config::load().expect("Failed to load config");
    info!("App config: {:?}", config);

    // Agent
    let agent_id = AgentId::new(&config.agent_label, config.id.clone());
    info!("Agent id: {:?}", &agent_id);
    let group = SharedGroup::new("loadbalancer", agent_id.as_account_id().clone());
    let (mut tx, rx) = AgentBuilder::new(agent_id.clone())
        .mode(ConnectionMode::Service)
        .start(&config.mqtt)
        .expect("Failed to create an agent");

    //
    let (ch_tx, mut ch_rx) = futures_channel::mpsc::unbounded::<Notification>();
    thread::spawn(move || {
        for message in rx {
            if let Err(e) = ch_tx.unbounded_send(message) {
                error!(
                    "Error sending message to the internal channel, {detail}",
                    detail = e
                );
            }
        }
    });

    // Application resources
    let state = Arc::new(State {
        message: endpoint::message::State::new(),
    });

    // Create Subscriptions
    tx.subscribe(
        &Subscription::multicast_requests(),
        QoS::AtLeastOnce,
        Some(&group),
    )
    .expect("Error subscribing to everyone's output messages");

    // Thread Pool
    let mut threadpool = ThreadPool::new()?;

    while let Some(message) = await!(ch_rx.next()) {
        let tx = tx.clone();
        let state = state.clone();
        threadpool.spawn(async move {
            let mut tx = tx.clone();
            match message {
                svc_agent::mqtt::Notification::Publish(message) => {
                    let topic: &str = &message.topic_name;

                    {
                        // Log incoming messages
                        let bytes = &message.payload.as_slice();
                        let text = std::str::from_utf8(bytes).unwrap_or("[non-utf8 characters]");
                        info!(
                            "Incoming message = '{}' sent to the topic = '{}'",
                            text, topic,
                        )
                    }

                    let result = await!(handle_message(
                        &mut tx,
                        message.payload.clone(),
                        state.clone(),
                    ));

                    if let Err(err) = result {
                        let bytes = &message.payload.as_slice();
                        let text = std::str::from_utf8(bytes).unwrap_or("[non-utf8 characters]");
                        error!(
                            "Error processing a message = '{text}' sent to the topic = '{topic}', '{detail}'",
                            text = text,
                            topic = topic,
                            detail = err,
                        )
                    }
                }
                _ => error!("An unsupported type of message = '{:?}'", message),
            }

        }).unwrap();
    }

    Ok(())
}

async fn handle_message(
    tx: &mut Agent,
    payload: Arc<Vec<u8>>,
    state: Arc<State>,
) -> Result<(), Error> {
    use endpoint::{handle_badrequest, handle_badrequest_method, handle_response};

    let envelope = serde_json::from_slice::<compat::IncomingEnvelope>(payload.as_slice())?;
    match envelope.properties() {
        compat::IncomingEnvelopeProperties::Request(ref reqp) => {
            let reqp = reqp.clone();
            match reqp.method() {
                method @ "message.echo" => {
                    let error_title = "Error handling an echo message";
                    match compat::into_request(envelope) {
                        Ok(req) => {
                            let next = await!(state.message.echo(req));
                            handle_response(method, error_title, tx, &reqp, next)
                        }
                        Err(err) => handle_badrequest(method, error_title, tx, &reqp, &err),
                    }
                }
                method => handle_badrequest_method(method, tx, &reqp),
            }
        }
        _ => Err(format_err!(
            "unsupported message type, envelope = '{:?}'",
            envelope
        )),
    }
}

mod config;
mod endpoint;
