use crate::auth::{create_magic_link, find_or_create_user, session_cookie_value, verify_magic_link};
use crate::models::MagicLinkRequest;
use askama::Template;
use axum::{
    extract::{Query, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    Form,
};
use serde::Deserialize;
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub base_url: String,
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {
    error: Option<String>,
    message: Option<String>,
}

pub async fn login_page() -> impl IntoResponse {
    let tmpl = LoginTemplate {
        error: None,
        message: None,
    };
    Html(tmpl.render().unwrap())
}

pub async fn request_magic_link(
    State(state): State<AppState>,
    Form(form): Form<MagicLinkRequest>,
) -> impl IntoResponse {
    match find_or_create_user(&state.pool, &form.email).await {
        Ok(user) => match create_magic_link(&state.pool, &user.id).await {
            Ok(token) => {
                let link = format!("{}/auth/verify?token={}", state.base_url, token);
                tracing::info!("Magic link for {}: {}", user.email, link);

                let tmpl = LoginTemplate {
                    error: None,
                    message: Some(format!(
                        "Magic link created (dev mode). Open: {}",
                        link
                    )),
                };
                Html(tmpl.render().unwrap()).into_response()
            }
            Err(e) => {
                let tmpl = LoginTemplate {
                    error: Some(e.to_string()),
                    message: None,
                };
                (StatusCode::INTERNAL_SERVER_ERROR, Html(tmpl.render().unwrap())).into_response()
            }
        },
        Err(e) => {
            let tmpl = LoginTemplate {
                error: Some(e.to_string()),
                message: None,
            };
            (StatusCode::BAD_REQUEST, Html(tmpl.render().unwrap())).into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct VerifyQuery {
    pub token: String,
}

pub async fn verify(
    State(state): State<AppState>,
    Query(q): Query<VerifyQuery>,
) -> Response {
    match verify_magic_link(&state.pool, &q.token).await {
        Ok(user) => {
            let cookie = format!(
                "session={}; Path=/; HttpOnly; SameSite=Lax; Max-Age=2592000",
                session_cookie_value(&user.id)
            );
            let mut res = Redirect::to("/board").into_response();
            res.headers_mut()
                .insert(header::SET_COOKIE, cookie.parse().unwrap());
            res
        }
        Err(e) => {
            let tmpl = LoginTemplate {
                error: Some(e.to_string()),
                message: None,
            };
            (StatusCode::UNAUTHORIZED, Html(tmpl.render().unwrap())).into_response()
        }
    }
}

pub async fn logout() -> impl IntoResponse {
    let cookie = "session=; Path=/; HttpOnly; Max-Age=0";
    let mut res = Redirect::to("/").into_response();
    res.headers_mut()
        .insert(header::SET_COOKIE, cookie.parse().unwrap());
    res
}

pub fn get_user_id_from_cookie(headers: &axum::http::HeaderMap) -> Option<String> {
    headers
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';').find_map(|c| {
                let c = c.trim();
                c.strip_prefix("session=").map(|v| v.to_string())
            })
        })
}
