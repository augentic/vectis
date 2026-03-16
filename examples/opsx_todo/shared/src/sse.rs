use std::{future, pin::Pin};

use async_sse::{Event as SseEvent, decode};
use crux_core::{Request, capability::Operation, command::StreamBuilder};
use facet::Facet;
use futures::{Stream, StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};

#[derive(Facet, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SseRequest {
    pub url: String,
}

#[derive(Facet, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[repr(C)]
pub enum SseResponse {
    Chunk(Vec<u8>),
    Done,
}

impl SseResponse {
    #[must_use]
    pub const fn is_done(&self) -> bool {
        matches!(self, Self::Done)
    }
}

impl Operation for SseRequest {
    type Output = SseResponse;
}

/// An SSE message carrying both the event type name and the raw JSON data.
#[derive(Facet, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SseMessage {
    pub event_type: String,
    pub data: String,
}

pub struct ServerSentEvents;

impl ServerSentEvents {
    pub fn get<Effect, Event>(
        url: impl Into<String>,
    ) -> StreamBuilder<Effect, Event, Pin<Box<dyn Stream<Item = SseMessage> + Send>>>
    where
        Effect: From<Request<SseRequest>> + Send + 'static,
        Event: Send + 'static,
    {
        let url = url.into();

        StreamBuilder::new(move |ctx| -> Pin<Box<dyn Stream<Item = SseMessage> + Send>> {
            let chunk_reader = ctx
                .stream_from_shell(SseRequest { url })
                .take_while(|response| future::ready(!response.is_done()))
                .map(|response| {
                    let SseResponse::Chunk(data) = response else {
                        unreachable!()
                    };
                    Ok::<_, std::io::Error>(data)
                })
                .into_async_read();

            Box::pin(
                decode(chunk_reader).filter_map(|sse_event| async {
                    sse_event.ok().and_then(|event| match event {
                        SseEvent::Message(msg) => {
                            let event_type = msg.name().clone();
                            let data =
                                String::from_utf8(msg.data().to_vec()).ok()?;
                            Some(SseMessage { event_type, data })
                        }
                        SseEvent::Retry(_) => None,
                    })
                }),
            )
        })
    }
}
