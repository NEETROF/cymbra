//! The user module's gRPC **server** adapter (task 3.5): exposes `UserService`
//! by translating each RPC into a [`UserPort`] call. The caller's identity comes
//! from the internal-token interceptor (request extension), never the body.

// tonic's `Status` makes `Result<_, Status>` large; this is unavoidable on the
// generated service signatures.
#![allow(clippy::result_large_err)]

use std::sync::Arc;

use cymbra_platform::AuthIdentity;
use cymbra_user_port::UserPort;
use cymbra_user_port::proto::{
    Account, DeleteAccountRequest, DeleteAccountResponse, GetAccountRequest, Identity,
    ListIdentitiesRequest, ListIdentitiesResponse, UpdateAccountRequest,
    user_service_server::{UserService, UserServiceServer},
};
use tonic::{Request, Response, Status};

/// Wraps the user port as a tonic `UserService`.
pub struct UserGrpc<P: UserPort> {
    port: Arc<P>,
}

impl<P: UserPort + 'static> UserGrpc<P> {
    pub fn new(port: Arc<P>) -> Self {
        Self { port }
    }

    /// Mountable tonic server.
    pub fn into_server(self) -> UserServiceServer<Self> {
        UserServiceServer::new(self)
    }
}

fn identity<T>(req: &Request<T>) -> Result<AuthIdentity, Status> {
    req.extensions()
        .get::<AuthIdentity>()
        .cloned()
        .ok_or_else(|| Status::unauthenticated("missing identity"))
}

fn to_proto(a: cymbra_user_port::Account) -> Account {
    Account {
        user_id: a.user_id,
        display_name: a.display_name,
        preferences: a.preferences,
        version: a.version,
        updated_at: a.updated_at,
    }
}

#[tonic::async_trait]
impl<P: UserPort + 'static> UserService for UserGrpc<P> {
    async fn get_account(
        &self,
        req: Request<GetAccountRequest>,
    ) -> Result<Response<Account>, Status> {
        let id = identity(&req)?;
        let acc = self.port.get_account(&id.user_id).await?;
        Ok(Response::new(to_proto(acc)))
    }

    async fn update_account(
        &self,
        req: Request<UpdateAccountRequest>,
    ) -> Result<Response<Account>, Status> {
        let id = identity(&req)?;
        let r = req.into_inner();
        let acc = self
            .port
            .update_account(
                &id.user_id,
                r.display_name,
                &r.preferences,
                r.expected_version,
            )
            .await?;
        Ok(Response::new(to_proto(acc)))
    }

    async fn list_identities(
        &self,
        req: Request<ListIdentitiesRequest>,
    ) -> Result<Response<ListIdentitiesResponse>, Status> {
        let id = identity(&req)?;
        let identities = self
            .port
            .list_identities(&id.user_id)
            .await?
            .into_iter()
            .map(|i| Identity {
                provider: i.provider,
                subject: i.subject,
                linked_at: i.linked_at,
            })
            .collect();
        Ok(Response::new(ListIdentitiesResponse { identities }))
    }

    async fn delete_account(
        &self,
        req: Request<DeleteAccountRequest>,
    ) -> Result<Response<DeleteAccountResponse>, Status> {
        let id = identity(&req)?;
        self.port.delete_account(&id.user_id).await?;
        Ok(Response::new(DeleteAccountResponse {}))
    }
}
