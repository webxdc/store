//! Local server to receive Githubs webhooks
use log::{error, info};
use std::sync::Arc;
use tide::{Request, Server as TideServer};
use tokio::sync::mpsc::Sender;
use crate::{shared::AppInfo, PORT};

#[derive(Clone)]
pub struct ServerState {
    pub channel: Arc<Sender<Vec<AppInfo>>>,
}

pub struct Server {
    server: TideServer<ServerState>,
}

async fn handler(mut req: Request<ServerState>) -> tide::Result {
    match receive_webhoook(&mut req).await {
        Ok(event) => {
            info!("received webhook");
            req.state().channel.send(event).await.unwrap();
        }
        Err(err) => error!("{err}"),
    };
    Ok("Success".into())
}

async fn get_handler(_req: Request<ServerState>) -> tide::Result {
    Ok("Hi".into())
}

impl Server {
    pub fn new(channel: Sender<Vec<AppInfo>>) -> Self {
        let mut server = tide::with_state(ServerState {
            channel: Arc::new(channel),
        });
        server.at("receive").post(handler).get(get_handler);
        Self { server }
    }

    pub fn start(&self) -> tokio::task::JoinHandle<()> {
        let server = self.server.clone();
        #[cfg(debug_assertions)]
        let handle = tokio::spawn(async move {
            server.listen(format!("127.0.0.1:{PORT}")).await.unwrap();
        });
        #[cfg(not(debug_assertions))]
        let handle = tokio::spawn(async move {
            server.listen(format!("0.0.0.0:{PORT}")).await.unwrap();
        });
        handle
    }

    pub fn stop(self) {}
}

async fn receive_webhoook(req: &mut Request<ServerState>) -> anyhow::Result<Vec<AppInfo>> {
    let body = req
        .take_body()
        .into_string()
        .await
        .map_err(anyhow::Error::msg)?;
    serde_json::from_str(body.as_str()).map_err(anyhow::Error::msg)
}
