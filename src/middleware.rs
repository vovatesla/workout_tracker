use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
    http::StatusCode,
};
use crate::auth;

pub async fn require_auth(
    State((_, jwt_secret, _, _)): State<(sqlx::PgPool, String, redis::Client, lapin::Channel)>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok());

    let token = match auth_header {
        Some(h) if h.starts_with("Bearer ") => {
            &h[7..]
        }
        _ => return Err(StatusCode::UNAUTHORIZED),
    };

    let claims = auth::verify_token(token, &jwt_secret)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    request.extensions_mut().insert(claims.sub);

    Ok(next.run(request).await)
}