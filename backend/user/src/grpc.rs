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
    Account, CheckHandleAvailabilityRequest, CheckHandleAvailabilityResponse, DeleteAccountRequest,
    DeleteAccountResponse, GetAccountRequest, GrantRoleRequest, GrantRoleResponse, Identity,
    ListIdentitiesRequest, ListIdentitiesResponse, ListRoleGrantsRequest, ListRoleGrantsResponse,
    RevokeRoleRequest, RevokeRoleResponse, RoleGrant as ProtoRoleGrant, UpdateAccountRequest,
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
        handle: a.handle,
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
                r.handle,
                &r.preferences,
                r.expected_version,
            )
            .await?;
        Ok(Response::new(to_proto(acc)))
    }

    async fn check_handle_availability(
        &self,
        req: Request<CheckHandleAvailabilityRequest>,
    ) -> Result<Response<CheckHandleAvailabilityResponse>, Status> {
        // Authenticated, but the answer is independent of the caller's account.
        let _ = identity(&req)?;
        let available = self
            .port
            .check_handle_availability(&req.into_inner().handle)
            .await?;
        Ok(Response::new(CheckHandleAvailabilityResponse { available }))
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

    async fn grant_role(
        &self,
        req: Request<GrantRoleRequest>,
    ) -> Result<Response<GrantRoleResponse>, Status> {
        // Admin-only (change: add-moderation-back-office). Because every grant is
        // gated on `admin`, granting the `admin` role also requires the caller to be
        // an admin. The acting admin is the authenticated caller, never the body.
        let id = identity(&req)?;
        cymbra_platform::guard::require_admin(&id)?;
        let r = req.into_inner();
        self.port
            .grant_role(&id.user_id, &r.user_id, &r.scope, &r.role)
            .await?;
        Ok(Response::new(GrantRoleResponse {}))
    }

    async fn revoke_role(
        &self,
        req: Request<RevokeRoleRequest>,
    ) -> Result<Response<RevokeRoleResponse>, Status> {
        let id = identity(&req)?;
        cymbra_platform::guard::require_admin(&id)?;
        let r = req.into_inner();
        self.port
            .revoke_role(&id.user_id, &r.user_id, &r.scope, &r.role)
            .await?;
        Ok(Response::new(RevokeRoleResponse {}))
    }

    async fn list_role_grants(
        &self,
        req: Request<ListRoleGrantsRequest>,
    ) -> Result<Response<ListRoleGrantsResponse>, Status> {
        let id = identity(&req)?;
        cymbra_platform::guard::require_admin(&id)?;
        let grants = self
            .port
            .list_role_grants(&req.into_inner().user_id)
            .await?
            .into_iter()
            .map(|g| ProtoRoleGrant {
                target_user_id: g.target_user_id,
                scope: g.scope,
                role: g.role,
                action: g.action,
                acting_admin: g.acting_admin,
                at: g.at,
            })
            .collect();
        Ok(Response::new(ListRoleGrantsResponse { grants }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FakeUserRepo, UserModule};
    use cymbra_user_port::UserPort;

    /// A `UserGrpc` over an in-memory module, returning the shared module handle so
    /// tests can assert role state directly.
    fn grpc() -> (
        UserGrpc<UserModule<FakeUserRepo>>,
        Arc<UserModule<FakeUserRepo>>,
    ) {
        let module = Arc::new(UserModule::new(FakeUserRepo::default()));
        (UserGrpc::new(module.clone()), module)
    }

    fn authed<T>(msg: T, user_id: &str, roles: &[&str]) -> Request<T> {
        let mut req = Request::new(msg);
        req.extensions_mut().insert(AuthIdentity {
            user_id: user_id.into(),
            audience: "music".into(),
            roles: roles.iter().map(|r| (*r).into()).collect(),
        });
        req
    }

    #[tokio::test]
    async fn grant_and_revoke_require_admin() {
        let (g, module) = grpc();
        let target = module.resolve_or_provision("google", "t").await.unwrap();
        let grant = || GrantRoleRequest {
            user_id: target.clone(),
            scope: "music".into(),
            role: "moderator".into(),
        };
        // A non-admin (plain user) is denied and nothing changes.
        let err = g
            .grant_role(authed(grant(), "u1", &["user"]))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::PermissionDenied);
        assert!(
            !module
                .effective_roles(&target, "music")
                .await
                .unwrap()
                .contains(&"moderator".to_string())
        );
        // An admin succeeds; the target gains the role.
        g.grant_role(authed(grant(), "admin1", &["user", "admin"]))
            .await
            .unwrap();
        assert!(
            module
                .effective_roles(&target, "music")
                .await
                .unwrap()
                .contains(&"moderator".to_string())
        );
        // Revoke is likewise admin-only.
        let revoke = RevokeRoleRequest {
            user_id: target.clone(),
            scope: "music".into(),
            role: "moderator".into(),
        };
        let err = g
            .revoke_role(authed(revoke, "u1", &["user"]))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::PermissionDenied);
    }

    #[tokio::test]
    async fn list_role_grants_is_admin_only_and_returns_audit() {
        let (g, module) = grpc();
        let target = module.resolve_or_provision("google", "t").await.unwrap();
        g.grant_role(authed(
            GrantRoleRequest {
                user_id: target.clone(),
                scope: "music".into(),
                role: "moderator".into(),
            },
            "admin1",
            &["user", "admin"],
        ))
        .await
        .unwrap();
        // Non-admin cannot read the audit.
        let err = g
            .list_role_grants(authed(
                ListRoleGrantsRequest {
                    user_id: target.clone(),
                },
                "u1",
                &["user"],
            ))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::PermissionDenied);
        // Admin reads the recorded grant.
        let resp = g
            .list_role_grants(authed(
                ListRoleGrantsRequest {
                    user_id: target.clone(),
                },
                "admin1",
                &["user", "admin"],
            ))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(resp.grants.len(), 1);
        assert_eq!(resp.grants[0].action, "grant");
        assert_eq!(resp.grants[0].role, "moderator");
    }
}
