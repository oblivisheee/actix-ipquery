# actix-ipquery
 
## Overview

`actix-ipquery` is an Actix Web middleware that allows you to query IP information using the `ipapi` crate and store the results using a custom store that implements the `IPQueryStore` trait. It supports querying the IP address from either the `X-Forwarded-For` header or the peer address of the request.

## Features

- Query IP information using a specified endpoint.
- Store IP information using a custom store.
- Option to use the `X-Forwarded-For` header for IP address extraction.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
actix-web = "4"
actix-ipquery = "*"
```

## Usage

Here is a basic example of how to use `actix-ipquery`:

```rust
use actix_web::{web, App, HttpServer};
use actix_ipquery::{IPQuery, IPQueryStore};

#[derive(Clone)]
struct MyStore;

impl IPQueryStore for MyStore {
    fn store(&self, ip_info: ipapi::IPInfo) -> Result<(), std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), std::io::Error>> + Send>> {
        println!("{:?}", ip_info);
        Box::pin(async { Ok(()) })
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(IPQuery::new(MyStore).finish())
            .route("/", web::get().to(|| async { "Hello, world!" }))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Configuration

You can configure the middleware to use the `X-Forwarded-For` header:

```rust
let ip_query = IPQuery::new(MyStore)
    .forwarded_for(true)
    .finish();
```

## License

This project is licensed under the MIT License.