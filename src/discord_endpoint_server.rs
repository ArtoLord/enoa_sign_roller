use std::{convert::Infallible, net::SocketAddr, sync::Arc};
use anyhow::{anyhow, Result};
use hyper_util::rt::TokioIo;
use log::error;
use serenity::{all::Interaction, interactions_endpoint::Verifier, json, Client};
use http_body_util::{BodyExt, Full};
use hyper::{body::{Bytes, Incoming}, server::conn::http1, service::service_fn, Request, Response};
use tokio::net::TcpListener;

use crate::{config::ServerConf, discord::Handler};

struct Server {
    handler: Handler,
    verifier: Verifier,
    client: Client
}

impl Server {
    async fn handle(&self, req: Request<Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
        let resp = self.handle_impl(req).await;

        if let Err(e) = resp {
            error!("Error while handling request: {}", e);

            let resp = Response::builder()
                .status(500)
                .body(Full::new(Bytes::new())).unwrap();

            return Ok(resp);
        }

        Ok(resp.unwrap())
    }

    async fn handle_impl(&self, req: Request<Incoming>) -> Result<Response<Full<Bytes>>> {
        let headers = req.headers().clone();

        let find_header = |name| Some(headers.iter().find(|h| h.0 == name)?.1.to_str());

        let signature = find_header("X-Signature-Ed25519").ok_or(anyhow!("missing signature header"))??;
        let timestamp = find_header("X-Signature-Timestamp").ok_or(anyhow!("missing timestamp header"))??;

        let body = req.collect().await?.to_bytes();
        if self.verifier.verify(signature, timestamp, &body).is_err() {
            let resp = Response::builder()
                .status(403)
                .body(Full::new(Bytes::new()))?;

            return Ok(resp);
        }

        let interaction = json::from_slice::<Interaction>(&body)?;

        let res = self.handler.handle_interaction(&self.client.http, interaction).await;
        Ok(Response::new(Full::new(json::to_vec(&res)?.into())))
    }
}

pub async fn start(handler: Handler, client: Client, config: ServerConf) -> Result<()> {
    handler.init_guilds(&client.http).await?;

    let addr: SocketAddr = config.address().parse()?;
    let listener = TcpListener::bind(addr).await?;

    let server = Arc::new(Server {
        handler: handler,
        verifier: Verifier::new(&config.discord_pk()),
        client: client
    });

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let server_ref = server.clone();

        // Spawn a tokio task to serve multiple connections concurrently
        tokio::task::spawn(async move {
            // Finally, we bind the incoming connection to our `hello` service
            if let Err(err) = http1::Builder::new()
                // `service_fn` converts our function in a `Service`
                .serve_connection(io, service_fn(|req| server_ref.handle(req)))
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}