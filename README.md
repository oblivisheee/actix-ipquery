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
use actix_ipquery::{IPInfo, IPQuery, IPQueryStore};
use actix_web::{App, HttpServer};
#[actix_web::main]
async fn main() {
    HttpServer::new(|| App::new().wrap(IPQuery::new(Store).finish()))
        .bind("127.0.0.1:8080")
        .unwrap()
        .run()
        .await
        .unwrap()
}

#[derive(Clone)]
struct Store;
impl IPQueryStore for Store {
    fn store(
        &self,
        ip_info: IPInfo,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(), std::io::Error>> + Send + 'static>,
    > {
        println!("{:?}", ip_info);
        Box::pin(async { Ok(()) })
    }
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