// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

extern crate tera;

use std::{cell::RefCell, rc::Rc};

use tera::{
    events::{Events, HttpEvent, HttpResponder},
    permissions::{
        events::event_http::{self, Path},
        Permissions,
    },
    Runtime,
};
use tokio::sync::mpsc::{self, Sender};
use utilities::{
    http::{Body, Request, Response},
    result::Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Create permitted resources
    let allow_list = [Path::from("/")];

    // Create permissions
    let permissions = Permissions::builder()
        .add_permissions(&[(event_http::HttpEvent::SendResponse, &allow_list)])?
        .build();

    // Create channels.
    let (response_tx, mut response_rx) = mpsc::channel::<Response<Body>>(2);

    // Recieve response on a separate task.
    tokio::spawn(async move {
        match response_rx.recv().await {
            Some(response) => {
                println!("Received response = {:?}", response);
            }
            None => {
                println!("No response recieved");
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
    const { request, respondWith } = httpEvent;

    ///// REQUEST
    // Log request.
    log.info("request =", request);

    // Read body.
    const buf = await request.body.readAll();
    log.info("request body decoded =", decode(buf));

    ///// RESPONSE
    // Send response.
    await respondWith(new Response("Sending something back"));
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
