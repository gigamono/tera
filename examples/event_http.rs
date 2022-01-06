// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

extern crate tera;

use std::{cell::RefCell, future, rc::Rc};

use futures_util::{StreamExt, TryStreamExt};
use tera::{
    events::{Events, HttpEvent, HttpResponder},
    permissions::{
        events::event_http::{self},
        Permissions,
    },
    Runtime,
};
use tokio::sync::mpsc::{self, Sender};
use utilities::{
    hyper::{Body, Request, Response},
    result::Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Create permissions
    let permissions = Permissions::builder()
        .add_permissions(&[event_http::HttpEvent::SendResponse])?
        .build();

    // Create channels.
    let (response_tx, mut response_rx) = mpsc::channel::<Response<Body>>(2);

    // Recieve response on a separate task.
    tokio::spawn(async move {
        match response_rx.recv().await {
            Some(mut response) => {
                let body = response.body_mut();

                let stream = body.into_stream();
                stream
                    .for_each(|item| {
                        println!("Item = {:?}", item);
                        future::ready(())
                    })
                    .await;
            }
            None => {
                println!("No response recieved");
            }
        };
    });

    // Create events.
    let events = create_http_events(Rc::new(response_tx))?;

    // Create a new runtime.
    let mut runtime = Runtime::with_events(permissions, events, false, Default::default()).await?;

    // Read main module code.
    let main_module_filename = "./examples/js/event_http.js";
    let main_module_code = r#"
    const { log, decode, encode, events: { http }, Response } = Tera;

    // Read body.
    const buf = await http.request.body.readAll();

    // Log body.
    log.info("request body decoded =", decode(buf));

    // Send random response.
    if (Math.random() < 0.5) {
      await sendFixedResponse();
    } else {
      await sendStreamingResponse();
    }

    // Sending response body with fixed content length.
    async function sendFixedResponse() {
      await http.respondWith(
        new Response('{ "message": "Hello beep boop!" }', {
          headers: { "Content-Type": "application/json" },
        })
      );
    }

    // Streaming response body with transfer-encoding chunked in Http/1.1 or streaming in H2
    async function sendStreamingResponse() {
      async function* iterator() {
        for (let i = 0; i < 20; i++) {
          yield encode(`index = ${i}\n`);
        }
      }

      await http.respondWith(new Response(iterator()));
    }
    "#;

    // Execute main module.
    runtime
        .execute_module(main_module_filename, main_module_code)
        .await
}

fn create_http_events(response_tx: Rc<Sender<Response<Body>>>) -> Result<Rc<RefCell<Events>>> {
    let request = Request::builder().body(Body::from("Hello world"))?;
    let responder = Rc::new(HttpResponder::new(response_tx));
    let http_event = HttpEvent::new(request, responder);

    Ok(Rc::new(RefCell::new(Events {
        http: Some(http_event),
    })))
}
