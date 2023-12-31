use shuttle_runtime::{CustomError, Error};
use std::net::SocketAddr;

/// A wrapper type for [serenity::Client] so we can implement [shuttle_runtime::Service] for it.
pub struct SerenityService(pub serenity::Client);

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for SerenityService {
    /// Takes the client that is returned by the user in their [shuttle_runtime::main] function
    /// and starts it.
    async fn bind(mut self, _addr: SocketAddr) -> Result<(), Error> {
        self.0.start_autosharded().await.map_err(CustomError::new)?;

        Ok(())
    }
}

impl From<serenity::Client> for SerenityService {
    fn from(router: serenity::Client) -> Self {
        Self(router)
    }
}

pub type ShuttleSerenity = Result<SerenityService, Error>;
