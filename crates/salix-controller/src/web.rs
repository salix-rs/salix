use anyhow::Result;
use axum::Router;
use axum::routing::get;

pub(crate) struct Web {}

impl Web {
    pub(crate) fn new() -> Self {
        Self {}
    }

    pub(crate) async fn run(&self) -> Result<()> {
        let app = Router::new().route("/", get(|| async { "Hello, World!" }));

        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;

        axum::serve(listener, app).await?;

        Ok(())
    }
}
