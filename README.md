# Chaser

Invoice chasing dashboard with live external API integrations.

## One-command start

```bash
./start.sh
```

or

```bash
python3 -m http.server 8765
```

Open **http://localhost:8765**

## External APIs integrated

| Feature              | Endpoint                     | Purpose                          |
|----------------------|------------------------------|----------------------------------|
| Live FX rates        | open.er-api.com/v6/latest/USD| Real-time USD→EUR conversion     |
| Send Chase           | httpbin.org/post             | Real external POST of chase data |
| Accounting Sync      | httpbin.org/get + local merge| Demonstrates external pull + merge|
| Connectivity test    | httpbin.org                  | One-click health check           |

All calls are live. No API keys required for the demo endpoints.

## Core features

- Board view with overdue totals + days-late badges
- Harbor & Co sample matching the original preview
- Add / Send Chase / Snooze / Mark paid
- Persistent localStorage
- Integration status bar
- Full Auth Lab (OAuth 2.0+PKCE, JWT, HMAC, mTLS)
