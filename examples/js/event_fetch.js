// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

addEventListener("fetch", (event) => {
  event.respondWith(new Response("Hello secure-runtime!"));
});
