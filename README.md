# Chaser

Invoice chasing dashboard.  
**Stack:** Rust · Axum · HTMX · SQLite · Magic Link auth

Get paid faster. Stop chasing invoices manually.

## Quick start

```bash
cp .env.example .env
cargo run
```

Open http://localhost:3000

### Magic link (dev mode)
Enter any email → the magic link is printed in the terminal **and** shown on the page.  
Click it to log in. No real email is sent yet.

## Features (v1)

- Magic-link authentication
- Persistent invoices in SQLite
- Board with overdue totals + days-late
- Add invoice
- One-click Chase / Mark Paid / Snooze
- Ready for PlanClose CSV import (next)

## Environment

| Variable       | Default                  | Description                |
|----------------|--------------------------|----------------------------|
| `DATABASE_URL` | `sqlite:chaser.db`       | SQLite connection string   |
| `BASE_URL`     | `http://localhost:3000`  | Used for magic-link URLs   |
| `PORT`         | `3000`                   | Listen port                |

## Project layout

```
src/
  main.rs
  auth.rs
  db.rs
  models.rs
  handlers/
migrations/
templates/
static/
```

## Next

- Real email sending for magic links (Resend / Postmark)
- PlanClose CSV import endpoint
- Deploy (Fly.io / Railway)
- Sequences + better chase templates
