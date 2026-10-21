use crate::handlers::auth::{get_user_id_from_cookie, AppState};
use crate::models::Invoice;
use askama::Template;
use axum::{
    extract::State,
    http::HeaderMap,
    response::{Html, IntoResponse, Redirect},
};

#[derive(Template)]
#[template(path = "board.html")]
struct BoardTemplate {
    invoices: Vec<InvoiceView>,
    total_overdue: String,
    user_email: String,
}

struct InvoiceView {
    id: String,
    customer: String,
    amount: String,
    due_date: String,
    days_late: i64,
    status: String,
    last_chase: String,
}

pub async fn board(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let user_id = match get_user_id_from_cookie(&headers) {
        Some(id) => id,
        None => return Redirect::to("/").into_response(),
    };

    let user = sqlx::query_as::<_, crate::models::User>("SELECT * FROM users WHERE id = ?")
        .bind(&user_id)
        .fetch_optional(&state.pool)
        .await
        .ok()
        .flatten();

    let user_email = user.map(|u| u.email).unwrap_or_else(|| "unknown".into());

    let rows = sqlx::query_as::<_, Invoice>(
        "SELECT * FROM invoices WHERE user_id = ? ORDER BY due_date ASC",
    )
    .bind(&user_id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let mut total_cents: i64 = 0;
    let invoices: Vec<InvoiceView> = rows
        .into_iter()
        .map(|inv| {
            let days = inv.days_late();
            if inv.status == "open" && days > 0 {
                total_cents += inv.amount_cents;
            }
            InvoiceView {
                id: inv.id,
                customer: inv.customer,
                amount: inv.amount_display(),
                due_date: inv.due_date,
                days_late: days,
                status: inv.status,
                last_chase: inv.last_chase_at.unwrap_or_else(|| "—".into()),
            }
        })
        .collect();

    let total_overdue = format!("{:.2} USD", total_cents as f64 / 100.0);

    let tmpl = BoardTemplate {
        invoices,
        total_overdue,
        user_email,
    };

    Html(tmpl.render().unwrap()).into_response()
}
