// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

extern crate tera;

use std::{cell::RefCell, rc::Rc};

use tera::{
    events::{Events, HttpEvent, HttpResponder},
    permissions::Permissions,
    Runtime,
};
use tokio::sync::mpsc::{self, Sender};
use utilities::{
    http::{Body, Request, Response},
    result::Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Create permissions.
    let permissions = Permissions::default();

    // Create channels.
    let (response_tx, mut response_rx) = mpsc::channel::<Response<Body>>(2);

    // Recieve response on a separate task.
    tokio::spawn(async move {
        match response_rx.recv().await {
            Some(response) => {
                println!("Received response = {:?}", response);
            }
            None => {
                println!("No response recieved response");
            }
        };
    });

    // Create events.
    let events = create_http_events(Rc::new(response_tx))?;

    // Create a new runtime.
    let mut runtime = Runtime::default_event(permissions, events).await?;

    // Read main module code.
    let main_module_filename = "./examples/js/event_http.js";
    let main_module_code = r#"
    const { Response, httpEvent } = sys;
    console.log(`request = ${httpEvent.request}`);
    httpEvent.respondWith(new Response("Hello world"));
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
