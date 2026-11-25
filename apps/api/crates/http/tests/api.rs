//! The API in-process (`tower::ServiceExt::oneshot`) against real Postgres.
//! Sign-in, Families, Invitations and Kids (ADR-0010, ADR-0011).

use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use kids_http::{router, AppState, Passwords};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;

struct App {
    router: Router,
}

struct Reply {
    status: StatusCode,
    set_cookie: Option<String>,
    json: Value,
}

impl Reply {
    /// The `session=…` pair to send back, from this reply's `Set-Cookie`.
    fn session(&self) -> String {
        let cookie = self.set_cookie.as_deref().expect("a Set-Cookie header");
        cookie.split(';').next().expect("pair").to_owned()
    }
}

impl App {
    fn new(pool: PgPool) -> Self {
        let state = AppState::new(pool, Passwords::with_cost(8, 1, 1).expect("params"));
        App {
            router: router(state),
        }
    }

    async fn send(
        &self,
        method: Method,
        path: &str,
        cookie: Option<&str>,
        body: Option<Value>,
    ) -> Reply {
        let mut request = Request::builder().method(method).uri(path);
        if let Some(cookie) = cookie {
            request = request.header(header::COOKIE, cookie);
        }
        let request = match body {
            Some(body) => request
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string())),
            None => request.body(Body::empty()),
        }
        .expect("request");
        let response = self
            .router
            .clone()
            .oneshot(request)
            .await
            .expect("response");
        let status = response.status();
        let set_cookie = response
            .headers()
            .get(header::SET_COOKIE)
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned);
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let json = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes)
                .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into_owned()))
        };
        Reply {
            status,
            set_cookie,
            json,
        }
    }

    async fn get(&self, path: &str, cookie: &str) -> Reply {
        self.send(Method::GET, path, Some(cookie), None).await
    }

    async fn post(&self, path: &str, cookie: &str, body: Value) -> Reply {
        self.send(Method::POST, path, Some(cookie), Some(body))
            .await
    }

    /// Signs up and returns the Session cookie.
    async fn sign_up(&self, email: &str, name: &str) -> String {
        let reply = self
            .send(
                Method::POST,
                "/api/users",
                None,
                Some(json!({ "email": email, "name": name, "password": "correct horse battery" })),
            )
            .await;
        assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.json);
        reply.session()
    }

    /// Signs up the founder and creates a Family. Returns (cookie, family id).
    async fn family(&self) -> (String, String) {
        let owner = self.sign_up("owner@example.com", "Jonas").await;
        let reply = self
            .post(
                "/api/families",
                &owner,
                json!({ "name": "The Kacs", "currency": "SEK" }),
            )
            .await;
        assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.json);
        let id = reply.json["id"].as_str().expect("id").to_owned();
        (owner, id)
    }

    /// Invites `email` with `role` and returns the Invitation link's secret.
    async fn invite(&self, owner: &str, family: &str, email: &str, role: &str) -> String {
        let reply = self
            .post(
                &format!("/api/families/{family}/invitations"),
                owner,
                json!({ "email": email, "role": role }),
            )
            .await;
        assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.json);
        let link = reply.json["link"].as_str().expect("link");
        link.strip_prefix("/join/")
            .expect("a /join/ link")
            .to_owned()
    }

    /// Invites, signs up and accepts. Returns the new Member's cookie.
    async fn join(&self, owner: &str, family: &str, email: &str, role: &str) -> String {
        let secret = self.invite(owner, family, email, role).await;
        let cookie = self.sign_up(email, "Adult").await;
        let reply = self
            .post(
                &format!("/api/invitations/{secret}/accept"),
                &cookie,
                json!({}),
            )
            .await;
        assert_eq!(reply.status, StatusCode::OK, "{}", reply.json);
        cookie
    }
}

// ---------- health ----------

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn health_reports_ok(pool: PgPool) {
    let app = App::new(pool);
    let reply = app.send(Method::GET, "/api/health", None, None).await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.json, json!({ "status": "ok" }));
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn unknown_route_is_not_found(pool: PgPool) {
    let app = App::new(pool);
    let reply = app.send(Method::GET, "/api/nope", None, None).await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
}

// ---------- signing up and in ----------

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn signing_up_starts_a_session_in_a_secure_cookie(pool: PgPool) {
    let app = App::new(pool);
    let reply = app
        .send(
            Method::POST,
            "/api/users",
            None,
            Some(json!({ "email": " Jonas@Example.com", "name": "Jonas", "password": "correct horse battery" })),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED);
    assert_eq!(reply.json["user"]["email"], "jonas@example.com");
    assert_eq!(reply.json["user"]["name"], "Jonas");
    let cookie = reply.set_cookie.clone().expect("cookie");
    for flag in [
        "HttpOnly",
        "Secure",
        "SameSite=Lax",
        "Path=/api",
        "Max-Age=2592000",
    ] {
        assert!(cookie.contains(flag), "{cookie} lacks {flag}");
    }

    let me = app.get("/api/me", &reply.session()).await;
    assert_eq!(me.status, StatusCode::OK);
    assert_eq!(me.json["user"]["email"], "jonas@example.com");
    assert_eq!(me.json["families"], json!([]));
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn sign_up_rejects_a_bad_email_a_short_password_and_a_taken_email(pool: PgPool) {
    let app = App::new(pool);
    let attempt = |email: &'static str, password: &'static str| {
        let app = &app;
        async move {
            app.send(
                Method::POST,
                "/api/users",
                None,
                Some(json!({ "email": email, "name": "Jonas", "password": password })),
            )
            .await
        }
    };
    let bad_email = attempt("jonas", "correct horse battery").await;
    assert_eq!(bad_email.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(bad_email.json["error"], "invalid_email");

    let short = attempt("jonas@example.com", "short").await;
    assert_eq!(short.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(short.json["error"], "password_too_short");

    assert_eq!(
        attempt("jonas@example.com", "correct horse battery")
            .await
            .status,
        StatusCode::CREATED
    );
    let taken = attempt("JONAS@example.com", "another long password").await;
    assert_eq!(taken.status, StatusCode::CONFLICT);
    assert_eq!(taken.json["error"], "email_taken");
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn signing_in_needs_the_right_password(pool: PgPool) {
    let app = App::new(pool);
    app.sign_up("jonas@example.com", "Jonas").await;

    let sign_in = |email: &'static str, password: &'static str| {
        let app = &app;
        async move {
            app.send(
                Method::POST,
                "/api/sessions",
                None,
                Some(json!({ "email": email, "password": password })),
            )
            .await
        }
    };
    let wrong = sign_in("jonas@example.com", "wrong password!").await;
    assert_eq!(wrong.status, StatusCode::UNAUTHORIZED);
    assert_eq!(wrong.json["error"], "wrong_email_or_password");
    assert!(wrong.set_cookie.is_none());

    let unknown = sign_in("nobody@example.com", "correct horse battery").await;
    assert_eq!(unknown.status, StatusCode::UNAUTHORIZED);
    assert_eq!(unknown.json["error"], "wrong_email_or_password");

    let right = sign_in("Jonas@example.com", "correct horse battery").await;
    assert_eq!(right.status, StatusCode::OK);
    assert_eq!(
        app.get("/api/me", &right.session()).await.status,
        StatusCode::OK
    );
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn signing_out_ends_the_session(pool: PgPool) {
    let app = App::new(pool);
    let cookie = app.sign_up("jonas@example.com", "Jonas").await;
    let out = app
        .send(Method::DELETE, "/api/sessions/current", Some(&cookie), None)
        .await;
    assert_eq!(out.status, StatusCode::NO_CONTENT);
    assert!(out.set_cookie.expect("cleared").contains("Max-Age=0"));
    let me = app.get("/api/me", &cookie).await;
    assert_eq!(me.status, StatusCode::UNAUTHORIZED);
    assert_eq!(me.json["error"], "not_signed_in");
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn without_a_session_nothing_is_visible(pool: PgPool) {
    let app = App::new(pool);
    assert_eq!(
        app.send(Method::GET, "/api/me", None, None).await.status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        app.get("/api/me", "session=forged").await.status,
        StatusCode::UNAUTHORIZED
    );
    let create = app
        .send(
            Method::POST,
            "/api/families",
            None,
            Some(json!({ "name": "X", "currency": "SEK" })),
        )
        .await;
    assert_eq!(create.status, StatusCode::UNAUTHORIZED);
}

// ---------- Families ----------

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn the_founder_sees_their_family_as_owner(pool: PgPool) {
    let app = App::new(pool);
    let (owner, family) = app.family().await;

    let me = app.get("/api/me", &owner).await;
    assert_eq!(
        me.json["families"],
        json!([{ "id": family, "name": "The Kacs", "role": "owner" }])
    );

    let view = app.get(&format!("/api/families/{family}"), &owner).await;
    assert_eq!(view.status, StatusCode::OK);
    assert_eq!(view.json["name"], "The Kacs");
    assert_eq!(view.json["currency"], "SEK");
    assert_eq!(view.json["you"]["role"], "owner");
    assert_eq!(view.json["members"][0]["name"], "Jonas");
    assert_eq!(view.json["kids"], json!([]));
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_family_is_invisible_to_non_members(pool: PgPool) {
    let app = App::new(pool);
    let (_, family) = app.family().await;
    let stranger = app.sign_up("stranger@example.com", "Stranger").await;
    let view = app.get(&format!("/api/families/{family}"), &stranger).await;
    assert_eq!(view.status, StatusCode::NOT_FOUND);
    let kid = app
        .post(
            &format!("/api/families/{family}/kids"),
            &stranger,
            json!({ "name": "Elsa" }),
        )
        .await;
    assert_eq!(kid.status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_family_needs_a_name_and_a_currency(pool: PgPool) {
    let app = App::new(pool);
    let owner = app.sign_up("owner@example.com", "Jonas").await;
    let reply = app
        .post(
            "/api/families",
            &owner,
            json!({ "name": " ", "currency": "SEK" }),
        )
        .await;
    assert_eq!(reply.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(reply.json["error"], "invalid_text");
    let reply = app
        .post(
            "/api/families",
            &owner,
            json!({ "name": "Kacs", "currency": "kronor" }),
        )
        .await;
    assert_eq!(reply.json["error"], "invalid_currency");
}

// ---------- Invitations ----------

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn an_invited_adult_previews_the_link_signs_up_and_joins(pool: PgPool) {
    let app = App::new(pool);
    let (owner, family) = app.family().await;
    let secret = app
        .invite(&owner, &family, "grandma@example.com", "viewer")
        .await;

    let preview = app
        .send(
            Method::GET,
            &format!("/api/invitations/{secret}"),
            None,
            None,
        )
        .await;
    assert_eq!(preview.status, StatusCode::OK);
    assert_eq!(preview.json["family_name"], "The Kacs");
    assert_eq!(preview.json["role"], "viewer");
    assert_eq!(preview.json["email"], "grandma@example.com");
    assert_eq!(preview.json["state"], "pending");

    let grandma = app.sign_up("grandma@example.com", "Mormor").await;
    let accepted = app
        .post(
            &format!("/api/invitations/{secret}/accept"),
            &grandma,
            json!({}),
        )
        .await;
    assert_eq!(accepted.status, StatusCode::OK);
    assert_eq!(accepted.json["family_id"], family.as_str());
    assert_eq!(accepted.json["role"], "viewer");

    let view = app.get(&format!("/api/families/{family}"), &grandma).await;
    assert_eq!(view.json["you"]["role"], "viewer");
    assert_eq!(view.json["members"].as_array().map(Vec::len), Some(2));
    // Pending Invitations are for Owners' eyes only.
    assert_eq!(view.json["invitations"], json!([]));

    let again = app
        .post(
            &format!("/api/invitations/{secret}/accept"),
            &grandma,
            json!({}),
        )
        .await;
    assert_eq!(again.status, StatusCode::CONFLICT);
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn an_invitation_is_only_accepted_by_its_email(pool: PgPool) {
    let app = App::new(pool);
    let (owner, family) = app.family().await;
    let secret = app
        .invite(&owner, &family, "grandma@example.com", "viewer")
        .await;
    let someone = app.sign_up("someone@example.com", "Someone").await;
    let reply = app
        .post(
            &format!("/api/invitations/{secret}/accept"),
            &someone,
            json!({}),
        )
        .await;
    assert_eq!(reply.status, StatusCode::FORBIDDEN);
    assert_eq!(reply.json["error"], "wrong_email");
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn an_unknown_link_is_not_found(pool: PgPool) {
    let app = App::new(pool);
    let reply = app
        .send(Method::GET, "/api/invitations/nope", None, None)
        .await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
    assert_eq!(reply.json["error"], "unknown_invitation");
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn owners_see_and_revoke_pending_invitations(pool: PgPool) {
    let app = App::new(pool);
    let (owner, family) = app.family().await;
    let secret = app
        .invite(&owner, &family, "grandma@example.com", "viewer")
        .await;
    let view = app.get(&format!("/api/families/{family}"), &owner).await;
    let invitation = &view.json["invitations"][0];
    assert_eq!(invitation["email"], "grandma@example.com");
    let id = invitation["id"].as_str().expect("id");

    let revoked = app
        .send(
            Method::DELETE,
            &format!("/api/families/{family}/invitations/{id}"),
            Some(&owner),
            None,
        )
        .await;
    assert_eq!(revoked.status, StatusCode::NO_CONTENT);
    let preview = app
        .send(
            Method::GET,
            &format!("/api/invitations/{secret}"),
            None,
            None,
        )
        .await;
    assert_eq!(preview.json["state"], "revoked");
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn only_owners_invite(pool: PgPool) {
    let app = App::new(pool);
    let (owner, family) = app.family().await;
    let parent = app
        .join(&owner, &family, "parent@example.com", "parent")
        .await;
    let reply = app
        .post(
            &format!("/api/families/{family}/invitations"),
            &parent,
            json!({ "email": "x@example.com", "role": "viewer" }),
        )
        .await;
    assert_eq!(reply.status, StatusCode::FORBIDDEN);
    assert_eq!(reply.json["error"], "not_allowed");
}

// ---------- Members ----------

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn an_owner_changes_roles_but_never_leaves_the_family_without_one(pool: PgPool) {
    let app = App::new(pool);
    let (owner, family) = app.family().await;
    let parent = app
        .join(&owner, &family, "parent@example.com", "parent")
        .await;
    let view = app.get(&format!("/api/families/{family}"), &owner).await;
    let me = view.json["you"]["member_id"]
        .as_str()
        .expect("id")
        .to_owned();
    let other = view.json["members"]
        .as_array()
        .expect("members")
        .iter()
        .find(|m| m["email"] == "parent@example.com")
        .and_then(|m| m["id"].as_str())
        .expect("parent")
        .to_owned();

    let step_down = app
        .send(
            Method::PATCH,
            &format!("/api/families/{family}/members/{me}"),
            Some(&owner),
            Some(json!({ "role": "parent" })),
        )
        .await;
    assert_eq!(step_down.status, StatusCode::CONFLICT);
    assert_eq!(step_down.json["error"], "last_owner");

    let promote = app
        .send(
            Method::PATCH,
            &format!("/api/families/{family}/members/{other}"),
            Some(&owner),
            Some(json!({ "role": "owner" })),
        )
        .await;
    assert_eq!(promote.status, StatusCode::NO_CONTENT);
    let view = app.get(&format!("/api/families/{family}"), &parent).await;
    assert_eq!(view.json["you"]["role"], "owner");
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_removed_member_no_longer_sees_the_family(pool: PgPool) {
    let app = App::new(pool);
    let (owner, family) = app.family().await;
    let viewer = app
        .join(&owner, &family, "viewer@example.com", "viewer")
        .await;
    let view = app.get(&format!("/api/families/{family}"), &viewer).await;
    let me = view.json["you"]["member_id"]
        .as_str()
        .expect("id")
        .to_owned();

    let left = app
        .send(
            Method::DELETE,
            &format!("/api/families/{family}/members/{me}"),
            Some(&viewer),
            None,
        )
        .await;
    assert_eq!(left.status, StatusCode::NO_CONTENT);
    let view = app.get(&format!("/api/families/{family}"), &viewer).await;
    assert_eq!(view.status, StatusCode::NOT_FOUND);
    assert_eq!(
        app.get("/api/me", &viewer).await.json["families"],
        json!([])
    );
}

// ---------- Kids ----------

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_parent_adds_a_kid_who_gets_an_empty_wallet(pool: PgPool) {
    let app = App::new(pool);
    let (owner, family) = app.family().await;
    let parent = app
        .join(&owner, &family, "parent@example.com", "parent")
        .await;
    let added = app
        .post(
            &format!("/api/families/{family}/kids"),
            &parent,
            json!({ "name": "Elsa" }),
        )
        .await;
    assert_eq!(added.status, StatusCode::CREATED, "{}", added.json);
    assert_eq!(added.json["name"], "Elsa");
    assert!(added.json["wallet_id"].is_string());

    let view = app.get(&format!("/api/families/{family}"), &owner).await;
    let kid = &view.json["kids"][0];
    assert_eq!(kid["name"], "Elsa");
    assert_eq!(kid["wallet_id"], added.json["wallet_id"]);
    assert_eq!(
        kid["wallet_total"],
        json!({ "minor": 0, "currency": "SEK" })
    );
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_viewer_cannot_add_kids_and_a_parent_archives_them(pool: PgPool) {
    let app = App::new(pool);
    let (owner, family) = app.family().await;
    let viewer = app
        .join(&owner, &family, "viewer@example.com", "viewer")
        .await;
    let refused = app
        .post(
            &format!("/api/families/{family}/kids"),
            &viewer,
            json!({ "name": "Elsa" }),
        )
        .await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN);

    let added = app
        .post(
            &format!("/api/families/{family}/kids"),
            &owner,
            json!({ "name": "Elsa" }),
        )
        .await;
    let kid = added.json["id"].as_str().expect("id");
    let archived = app
        .post(
            &format!("/api/families/{family}/kids/{kid}/archive"),
            &owner,
            json!({}),
        )
        .await;
    assert_eq!(archived.status, StatusCode::NO_CONTENT);
    let view = app.get(&format!("/api/families/{family}"), &owner).await;
    assert_eq!(view.json["kids"], json!([]));
}
