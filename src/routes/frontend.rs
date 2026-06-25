use askama::Template;
use axum::{
    Form, Router,
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use serde::Deserialize;

use crate::{
    app::AppState,
    auth::user::{UnauthenticatedUser, User},
    error::AppError,
    models::Asset,
    repository::Repository,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/login", get(login_page).post(login))
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginPage;

#[derive(Template)]
#[template(path = "dashboard.html")]
struct DashboardPage {
    username: String,
    assets: Vec<Asset>,
    total: f64,
}

async fn login_page() -> Result<Html<String>, AppError> {
    let html = LoginPage.render()?;
    Ok(Html(html))
}

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

async fn login(
    repository: Repository,
    jar: CookieJar,
    Form(request): Form<LoginForm>,
) -> Result<impl IntoResponse, AppError> {
    let unauth_user = UnauthenticatedUser::new(request.username, request.password);
    let user = match unauth_user.authenticate(&repository).await {
        Ok(user) => user,
        Err(AppError::UserDoesNotExist) => unauth_user.register(&repository).await?,
        Err(other_err) => return Err(other_err),
    };

    let token = user.auth_token()?;
    let cookie = Cookie::build(("token", token)).http_only(true);

    Ok((jar.add(cookie), Redirect::to("/")))
}

async fn index(
    maybe_user: Option<User>,
    repository: Repository,
) -> Result<Response, AppError> {
    match maybe_user {
        Some(user) => {
            let assets = repository.list_assets().await?;
            let total = repository.total_value().await?;
            let page = DashboardPage {
                username: user.username().to_string(),
                assets,
                total,
            };
            let html = page.render()?;
            Ok(Html(html).into_response())
        }
        None => Ok(Redirect::to("/login").into_response()),
    }
}