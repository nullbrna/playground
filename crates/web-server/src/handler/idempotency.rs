use axum::extract::State;
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use sqlx::query;
use sqlx::query_scalar;
use tracing::info;

use crate::AppState;
use crate::handler::HandlerResult;

pub async fn core(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> HandlerResult<impl IntoResponse> {
    let idempotency_key = if let Some(header_value) = headers.get("Idempotency-Key") {
        header_value.to_str().map_err(anyhow::Error::from)?
    } else {
        return Err(StatusCode::BAD_REQUEST.into());
    };

    let statement = r#"
        SELECT status
        FROM idempotency
        WHERE key = $1
        AND NOW() < expires_at
    "#;

    let cached: Option<i16> = query_scalar(statement)
        .bind(idempotency_key)
        .fetch_optional(&state.pool)
        .await?;

    if let Some(cached_status) = cached {
        let status_code = u16::try_from(cached_status).map_err(anyhow::Error::from)?;
        let status_code = StatusCode::from_u16(status_code).map_err(anyhow::Error::from)?;

        info!("[IDEMPOTENCY] cache hit: {idempotency_key}");
        return Ok((status_code, "CACHE_HIT"));
    };

    let statement = r#"
        INSERT INTO idempotency (key, status, expires_at)
        VALUES ($1, $2, NOW() + MAKE_INTERVAL(SECS => $3))
    "#;

    query(statement)
        .bind(idempotency_key)
        .bind(201)
        .bind(86400) // 1 day TTL in seconds
        .execute(&state.pool)
        .await?;

    info!("[IDEMPOTENCY] cache miss: {idempotency_key}");
    Ok((StatusCode::CREATED, "CACHE_MISS"))
}
