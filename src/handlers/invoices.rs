use crate::handlers::auth::{get_user_id_from_cookie, AppState};
use crate::models::{ChaseRequest, NewInvoice};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect},
    Form,
};
use chrono::Utc;
use uuid::Uuid;

pub async fn create_invoice(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<NewInvoice>,
) -> impl IntoResponse {
    let user_id = match get_user_id_from_cookie(&headers) {
        Some(id) => id,
        None => return Redirect::to("/").into_response(),
    };

    let amount_cents = (form.amount * 100.0).round() as i64;
    let currency = form.currency.unwrap_or_else(|| "USD".into());
    let id = Uuid::new_v4().to_string();

    let res = sqlx::query(
        r#"
        INSERT INTO invoices (id, user_id, customer, amount_cents, currency, due_date, notes)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&user_id)
    .bind(&form.customer)
    .bind(amount_cents)
    .bind(&currency)
    .bind(&form.due_date)
    .bind(&form.notes)
    .execute(&state.pool)
    .await;

    match res {
        Ok(_) => Redirect::to("/board").into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create invoice: {}", e),
        )
            .into_response(),
    }
}

pub async fn send_chase(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Form(form): Form<ChaseRequest>,
) -> impl IntoResponse {
    let user_id = match get_user_id_from_cookie(&headers) {
        Some(id) => id,
        None => return Redirect::to("/").into_response(),
    };

    let inv = sqlx::query_as::<_, crate::models::Invoice>(
        "SELECT * FROM invoices WHERE id = ? AND user_id = ?",
    )
    .bind(&id)
    .bind(&user_id)
    .fetch_optional(&state.pool)
    .await
    .ok()
    .flatten();

    if inv.is_none() {
        return (StatusCode::NOT_FOUND, "Invoice not found").into_response();
    }

    let chase_type = form.chase_type.unwrap_or_else(|| "friendly".into());
    let chase_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let _ = sqlx::query(
        "INSERT INTO chases (id, invoice_id, chase_type) VALUES (?, ?, ?)",
    )
    .bind(&chase_id)
    .bind(&id)
    .bind(&chase_type)
    .execute(&state.pool)
    .await;

    let _ = sqlx::query(
        "UPDATE invoices SET last_chase_at = ?, updated_at = ? WHERE id = ?",
    )
    .bind(&now)
    .bind(&now)
    .bind(&id)
    .execute(&state.pool)
    .await;

    Redirect::to("/board").into_response()
}

pub async fn mark_paid(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let user_id = match get_user_id_from_cookie(&headers) {
        Some(id) => id,
        None => return Redirect::to("/").into_response(),
    };

    let now = Utc::now().to_rfc3339();
    let _ = sqlx::query(
        "UPDATE invoices SET status = 'paid', updated_at = ? WHERE id = ? AND user_id = ?",
    )
    .bind(&now)
    .bind(&id)
    .bind(&user_id)
    .execute(&state.pool)
    .await;

    Redirect::to("/board").into_response()
}

pub async fn snooze(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let user_id = match get_user_id_from_cookie(&headers) {
        Some(id) => id,
        None => return Redirect::to("/").into_response(),
    };

    let snooze_until = (Utc::now() + chrono::Duration::days(7))
        .date_naive()
        .format("%Y-%m-%d")
        .to_string();
    let now = Utc::now().to_rfc3339();

    let _ = sqlx::query(
        "UPDATE invoices SET status = 'snoozed', snooze_until = ?, updated_at = ? WHERE id = ? AND user_id = ?",
    )
    .bind(&snooze_until)
    .bind(&now)
    .bind(&id)
    .bind(&user_id)
    .execute(&state.pool)
    .await;

    Redirect::to("/board").into_response()
}
