use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: String,
    pub email: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MagicLink {
    pub token: String,
    pub user_id: String,
    pub expires_at: String,
    pub used: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Invoice {
    pub id: String,
    pub user_id: String,
    pub customer: String,
    pub amount_cents: i64,
    pub currency: String,
    pub due_date: String,
    pub status: String,
    pub last_chase_at: Option<String>,
    pub snooze_until: Option<String>,
    pub notes: Option<String>,
    pub external_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Chase {
    pub id: String,
    pub invoice_id: String,
    pub chase_type: String,
    pub subject: Option<String>,
    pub body: Option<String>,
    pub sent_at: String,
}

#[derive(Debug, Deserialize)]
pub struct MagicLinkRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct NewInvoice {
    pub customer: String,
    pub amount: f64,
    pub currency: Option<String>,
    pub due_date: String,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChaseRequest {
    pub chase_type: Option<String>,
}

impl Invoice {
    pub fn amount_display(&self) -> String {
        format!("{:.2} {}", self.amount_cents as f64 / 100.0, self.currency)
    }

    pub fn days_late(&self) -> i64 {
        if self.status != "open" {
            return 0;
        }
        let due = chrono::NaiveDate::parse_from_str(&self.due_date, "%Y-%m-%d")
            .unwrap_or_else(|_| chrono::Utc::now().date_naive());
        let today = chrono::Utc::now().date_naive();
        (today - due).num_days().max(0)
    }
}
