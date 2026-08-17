//! The user module's gRPC **server** adapter (task 3.5): exposes `UserService`
//! by translating each RPC into a [`UserPort`] call. The caller's identity comes
//! from the internal-token interceptor (request extension), never the body.

// tonic's `Status` makes `Result<_, Status>` large; this is unavoidable on the
// generated service signatures.
#![allow(clippy::result_large_err)]

use std::sync::Arc;

use cymbra_platform::AuthIdentity;
use cymbra_user_port::proto::{
    Account, AccountRow, CheckHandleAvailabilityRequest, CheckHandleAvailabilityResponse,
    DeleteAccountRequest, DeleteAccountResponse, GetAccountRequest, GetPlayerProfileRequest,
    GrantRoleRequest, GrantRoleResponse, Identity, ListAccountsRequest, ListAccountsResponse,
    ListIdentitiesRequest, ListIdentitiesResponse, ListRoleGrantsRequest, ListRoleGrantsResponse,
    PlayerProfile as ProtoPlayerProfile, RevokeRoleRequest, RevokeRoleResponse,
    RoleGrant as ProtoRoleGrant, ScopeRoles as ProtoScopeRoles, SetLocaleRequest,
    SetProfileVisibilityRequest, SetProfileVisibilityResponse, UpdateAccountRequest,
    user_service_server::{UserService, UserServiceServer},
};
use cymbra_user_port::{UserPort, Visibility};
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
        locale: a.locale,
    }
}

fn to_proto_profile(p: cymbra_user_port::PlayerProfile) -> ProtoPlayerProfile {
    ProtoPlayerProfile {
        user_id: p.user_id,
        handle: p.handle,
        display_name: p.display_name,
        visibility: p.visibility.as_str().to_string(),
    }
}

/// Parse an ISO `yyyy-mm-dd` date-of-birth from the request (age gate).
fn parse_dob(s: &str) -> Result<chrono::NaiveDate, Status> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map_err(|_| Status::invalid_argument("date_of_birth must be ISO yyyy-mm-dd"))
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

    async fn set_locale(
        &self,
        req: Request<SetLocaleRequest>,
    ) -> Result<Response<Account>, Status> {
        // The account written is always the authenticated caller's — the locale
        // comes from the body, the target from the token (change: sync-account-
        // language-preference). `set_locale` is a no-op on empty input.
        let id = identity(&req)?;
        let locale = req.into_inner().locale;
        self.port.set_locale(&id.user_id, &locale).await?;
        let acc = self.port.get_account(&id.user_id).await?;
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
        // Scope-matched admin (change: scope-aware-role-admin): granting a role in
        // scope `S` requires `admin` in `S` or `global/admin`. A `music/admin` can no
        // longer touch `live`, and granting `admin` itself is gated the same way. The
        // acting admin is the authenticated caller, never the body.
        let id = identity(&req)?;
        let r = req.into_inner();
        cymbra_platform::guard::require_admin_in_scope(&id, &r.scope)?;
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
        let r = req.into_inner();
        cymbra_platform::guard::require_admin_in_scope(&id, &r.scope)?;
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
                acting_admin_handle: g.acting_admin_handle,
            })
            .collect();
        Ok(Response::new(ListRoleGrantsResponse { grants }))
    }

    async fn list_accounts(
        &self,
        req: Request<ListAccountsRequest>,
    ) -> Result<Response<ListAccountsResponse>, Status> {
        // Admin-only, scope-matched (change: scope-aware-role-admin): the directory
        // reveals who holds which role, so it exposes only the scopes the caller may
        // administer. A caller who is admin in no scope is refused, exactly like the
        // grant/revoke this feeds.
        let id = identity(&req)?;
        let scopes = id.admin_scopes(&crate::module::SCOPES);
        if scopes.is_empty() {
            return Err(Status::permission_denied("requires `admin` in a scope"));
        }
        let r = req.into_inner();
        let page = self
            .port
            .list_accounts_filtered(
                &cymbra_user_port::AccountFilter {
                    query: r.query,
                    ids: r.ids,
                    exclude_ids: r.exclude_ids,
                },
                r.limit as i64,
                r.offset as i64,
                &scopes,
            )
            .await?;
        let accounts = page
            .entries
            .into_iter()
            .map(|a| AccountRow {
                user_id: a.user_id,
                handle: a.handle,
                display_name: a.display_name,
                roles_by_scope: a
                    .roles_by_scope
                    .into_iter()
                    .map(|sr| ProtoScopeRoles {
                        scope: sr.scope,
                        roles: sr.roles,
                    })
                    .collect(),
            })
            .collect();
        Ok(Response::new(ListAccountsResponse {
            accounts,
            total: page.total as u32,
        }))
    }

    async fn get_player_profile(
        &self,
        req: Request<GetPlayerProfileRequest>,
    ) -> Result<Response<ProtoPlayerProfile>, Status> {
        // Authenticated viewers only; the port enforces visibility + eligibility
        // fail-closed (a private/ineligible target is `NotFound` to non-owners).
        let id = identity(&req)?;
        let target = req.into_inner().user_id;
        let today = chrono::Utc::now().date_naive();
        let profile = self
            .port
            .get_player_profile(&id.user_id, &target, today)
            .await?;
        Ok(Response::new(to_proto_profile(profile)))
    }

    async fn set_profile_visibility(
        &self,
        req: Request<SetProfileVisibilityRequest>,
    ) -> Result<Response<SetProfileVisibilityResponse>, Status> {
        let id = identity(&req)?;
        let r = req.into_inner();
        let visibility = Visibility::parse(&r.visibility)?;
        let dob = match r.date_of_birth {
            Some(s) => Some(parse_dob(&s)?),
            None => None,
        };
        let today = chrono::Utc::now().date_naive();
        let now = self
            .port
            .set_profile_visibility(&id.user_id, visibility, dob, today)
            .await?;
        Ok(Response::new(SetProfileVisibilityResponse {
            visibility: now.as_str().to_string(),
        }))
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

    /// A back-office request whose caller holds `roles` **in the `music` scope**
    /// (the common case for these tests). Scope-matched auth treats it as a
    /// `music`-scope caller.
    fn authed<T>(msg: T, user_id: &str, roles: &[&str]) -> Request<T> {
        authed_scoped(msg, user_id, &[("music", roles)])
    }

    /// A back-office request whose caller holds the given roles per scope — used to
    /// exercise scope-matched authorization across `global`/`music`/`live`.
    fn authed_scoped<T>(msg: T, user_id: &str, pairs: &[(&str, &[&str])]) -> Request<T> {
        let roles_by_scope: std::collections::BTreeMap<String, Vec<String>> = pairs
            .iter()
            .map(|(s, rs)| (s.to_string(), rs.iter().map(|r| (*r).to_string()).collect()))
            .collect();
        let mut roles: Vec<String> = Vec::new();
        for rs in roles_by_scope.values() {
            for r in rs {
                if !roles.contains(r) {
                    roles.push(r.clone());
                }
            }
        }
        let mut req = Request::new(msg);
        req.extensions_mut().insert(AuthIdentity {
            user_id: user_id.into(),
            audience: "back-office".into(),
            roles,
            roles_by_scope,
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

    #[tokio::test]
    async fn list_accounts_is_admin_only_and_filters() {
        let (g, module) = grpc();
        let target = module
            .resolve_or_provision("local", "ada@x.dev")
            .await
            .unwrap();
        module
            .update_account(&target, None, Some("ada".into()), "{}", 1)
            .await
            .unwrap();
        // A moderator (non-admin) is refused.
        let err = g
            .list_accounts(authed(
                ListAccountsRequest {
                    limit: 25,
                    offset: 0,
                    query: String::new(),
                    ..Default::default()
                },
                "u1",
                &["user", "moderator"],
            ))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::PermissionDenied);
        // An admin lists and can filter by handle.
        let resp = g
            .list_accounts(authed(
                ListAccountsRequest {
                    limit: 25,
                    offset: 0,
                    query: "ada".into(),
                    ..Default::default()
                },
                "admin1",
                &["user", "admin"],
            ))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(resp.total, 1);
        assert_eq!(resp.accounts[0].handle.as_deref(), Some("ada"));
    }

    #[tokio::test]
    async fn grant_is_scope_matched_across_scopes() {
        let (g, module) = grpc();
        let target = module.resolve_or_provision("google", "t").await.unwrap();
        let live_grant = || GrantRoleRequest {
            user_id: target.clone(),
            scope: "live".into(),
            role: "moderator".into(),
        };

        // A music-only admin cannot touch the `live` scope.
        let err = g
            .grant_role(authed_scoped(live_grant(), "m", &[("music", &["admin"])]))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::PermissionDenied);
        assert!(
            !module
                .effective_roles(&target, "live")
                .await
                .unwrap()
                .contains(&"moderator".to_string())
        );

        // A global admin (break-glass) can grant in `live`.
        g.grant_role(authed_scoped(live_grant(), "g", &[("global", &["admin"])]))
            .await
            .unwrap();
        assert!(
            module
                .effective_roles(&target, "live")
                .await
                .unwrap()
                .contains(&"moderator".to_string())
        );
    }

    #[tokio::test]
    async fn global_role_is_grantable_only_by_global_admin() {
        let (g, module) = grpc();
        let target = module.resolve_or_provision("google", "t").await.unwrap();
        let global_grant = || GrantRoleRequest {
            user_id: target.clone(),
            scope: "global".into(),
            role: "admin".into(),
        };

        // A music admin cannot grant in the `global` scope.
        let err = g
            .grant_role(authed_scoped(global_grant(), "m", &[("music", &["admin"])]))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::PermissionDenied);

        // A global admin can promote another account to `global/admin`.
        g.grant_role(authed_scoped(
            global_grant(),
            "g",
            &[("global", &["admin"])],
        ))
        .await
        .unwrap();
        assert!(
            module
                .effective_roles(&target, "music")
                .await
                .unwrap()
                .contains(&"admin".to_string())
        );
    }

    #[tokio::test]
    async fn directory_exposes_only_authorized_scopes() {
        let (g, module) = grpc();
        let target = module
            .resolve_or_provision("local", "t@x.dev")
            .await
            .unwrap();
        module
            .update_account(&target, None, Some("tara".into()), "{}", 1)
            .await
            .unwrap();
        // Target holds a role in each app scope.
        module
            .grant_role("seed", &target, "music", "moderator")
            .await
            .unwrap();
        module
            .grant_role("seed", &target, "live", "moderator")
            .await
            .unwrap();

        // A music-only admin sees a `music` column but no `live` roles at all.
        let resp = g
            .list_accounts(authed_scoped(
                ListAccountsRequest {
                    limit: 25,
                    offset: 0,
                    query: "tara".into(),
                    ..Default::default()
                },
                "m",
                &[("music", &["admin"])],
            ))
            .await
            .unwrap()
            .into_inner();
        let scopes: Vec<&str> = resp.accounts[0]
            .roles_by_scope
            .iter()
            .map(|sr| sr.scope.as_str())
            .collect();
        assert_eq!(scopes, vec!["music"]);

        // A global admin sees all three scopes.
        let resp = g
            .list_accounts(authed_scoped(
                ListAccountsRequest {
                    limit: 25,
                    offset: 0,
                    query: "tara".into(),
                    ..Default::default()
                },
                "gg",
                &[("global", &["admin"])],
            ))
            .await
            .unwrap()
            .into_inner();
        let mut scopes: Vec<&str> = resp.accounts[0]
            .roles_by_scope
            .iter()
            .map(|sr| sr.scope.as_str())
            .collect();
        scopes.sort();
        assert_eq!(scopes, vec!["global", "live", "music"]);

        // A caller who is admin in no scope is refused outright.
        let err = g
            .list_accounts(authed_scoped(
                ListAccountsRequest {
                    limit: 25,
                    offset: 0,
                    query: String::new(),
                    ..Default::default()
                },
                "u",
                &[("music", &["moderator"])],
            ))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::PermissionDenied);
    }

    #[tokio::test]
    async fn set_visibility_then_view_profile_end_to_end() {
        let (g, module) = grpc();
        let owner = module.resolve_or_provision("google", "o").await.unwrap();
        module
            .update_account(&owner, None, Some("ada".into()), "{}", 1)
            .await
            .unwrap();
        // Owner opts in to public with an eligible DOB.
        let resp = g
            .set_profile_visibility(authed(
                SetProfileVisibilityRequest {
                    visibility: "public".into(),
                    date_of_birth: Some("2000-01-01".into()),
                },
                &owner,
                &["user"],
            ))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(resp.visibility, "public");
        // Another authenticated player reads the allow-listed public profile.
        let p = g
            .get_player_profile(authed(
                GetPlayerProfileRequest {
                    user_id: owner.clone(),
                },
                "viewer",
                &["user"],
            ))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(p.handle.as_deref(), Some("ada"));
        assert_eq!(p.visibility, "public");
    }

    #[tokio::test]
    async fn private_profile_is_not_found_to_others() {
        let (g, module) = grpc();
        let owner = module.resolve_or_provision("google", "o").await.unwrap();
        let err = g
            .get_player_profile(authed(
                GetPlayerProfileRequest {
                    user_id: owner.clone(),
                },
                "viewer",
                &["user"],
            ))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::NotFound);
    }

    #[tokio::test]
    async fn profile_rpcs_reject_unauthenticated() {
        let (g, _module) = grpc();
        // No identity extension → unauthenticated on both new RPCs.
        let err = g
            .get_player_profile(Request::new(GetPlayerProfileRequest {
                user_id: "u".into(),
            }))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::Unauthenticated);
        let err = g
            .set_profile_visibility(Request::new(SetProfileVisibilityRequest {
                visibility: "public".into(),
                date_of_birth: None,
            }))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::Unauthenticated);
    }

    // --- Preferred locale (change: sync-account-language-preference) -----------

    #[tokio::test]
    async fn set_locale_writes_caller_account_and_get_account_returns_it() {
        let (g, module) = grpc();
        let uid = module.resolve_or_provision("google", "g1").await.unwrap();
        // No locale yet → GetAccount reports none.
        let acc = g
            .get_account(authed(GetAccountRequest {}, &uid, &["user"]))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(acc.locale, None);
        // SetLocale writes the caller's account and echoes the updated account.
        let echoed = g
            .set_locale(authed(
                SetLocaleRequest {
                    locale: "fr".into(),
                },
                &uid,
                &["user"],
            ))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(echoed.locale.as_deref(), Some("fr"));
        // A subsequent GetAccount reflects the stored value.
        let acc = g
            .get_account(authed(GetAccountRequest {}, &uid, &["user"]))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(acc.locale.as_deref(), Some("fr"));
    }

    #[tokio::test]
    async fn set_locale_is_a_noop_on_empty_input() {
        let (g, module) = grpc();
        let uid = module.resolve_or_provision("google", "g1").await.unwrap();
        g.set_locale(authed(
            SetLocaleRequest {
                locale: "fr".into(),
            },
            &uid,
            &["user"],
        ))
        .await
        .unwrap();
        // An empty locale leaves the stored value unchanged.
        let echoed = g
            .set_locale(authed(
                SetLocaleRequest {
                    locale: String::new(),
                },
                &uid,
                &["user"],
            ))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(echoed.locale.as_deref(), Some("fr"));
    }

    #[tokio::test]
    async fn set_locale_targets_only_the_caller_identity() {
        let (g, module) = grpc();
        let a = module.resolve_or_provision("google", "a").await.unwrap();
        let b = module.resolve_or_provision("google", "b").await.unwrap();
        // The target is the token identity — there is no account id in the body to
        // spoof, so each caller writes only their own account.
        g.set_locale(authed(
            SetLocaleRequest {
                locale: "fr".into(),
            },
            &a,
            &["user"],
        ))
        .await
        .unwrap();
        g.set_locale(authed(
            SetLocaleRequest {
                locale: "es".into(),
            },
            &b,
            &["user"],
        ))
        .await
        .unwrap();
        assert_eq!(
            module.get_account(&a).await.unwrap().locale.as_deref(),
            Some("fr")
        );
        assert_eq!(
            module.get_account(&b).await.unwrap().locale.as_deref(),
            Some("es")
        );
    }

    #[tokio::test]
    async fn set_locale_rejects_unauthenticated() {
        let (g, _module) = grpc();
        let err = g
            .set_locale(Request::new(SetLocaleRequest {
                locale: "fr".into(),
            }))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::Unauthenticated);
    }
}
