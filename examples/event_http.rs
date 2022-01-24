// Copyright 2021 the Gigamono authors. All rights reserved. GPL-3.0 License.

extern crate tera;

use std::{cell::RefCell, future, rc::Rc, fs};

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
        .add_permissions(&[event_http::HttpEvent::ResponseSend])?
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
    let mut runtime =
        Runtime::with_events(permissions, events, false, vec![], Default::default()).await?;

    // Read main module code.
    let code = fs::read_to_string("examples/js/event_http.js")?;

    // Execute main module.
    runtime
        .execute_module("/examples/js/event_http.js", code)
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
