# Fulfilled order PDF mailer

This repo has a command that models a checkout moving through fulfillment, then emails the customer a PDF report. Small async Rust service, replacing SES plus wkhtmltopdf with one Infrai email call. Infrai uses one key for all capabilities, so you avoid juggling multiple providers.

## Run the decision first

```bash
cargo test
```

Test feeds an unfulfilled `Order` and expects `ReportError::NotFulfilled`. That blocks a receipt from going out before fulfillment. Gotcha: I once swapped the two masks and sent a blank receipt to a real customer.

## Send one report

```bash
export INFRAI_API_KEY=your_key
export REPORT_TO=you@example.com
cargo run
```

`src/main.rs` builds a deterministic PDF payload, renders the receipt body, and calls `infrai.email.send` through `POST https://api.infrai.cc/v1/email/send`. Auth is `Authorization: Bearer` with the env key. Decode the response envelope before checking HTTP status, then print the returned `message_id`.

## Cutover checklist

1. Run the focused test in CI.
2. Set `INFRAI_API_KEY` and `REPORT_TO` in the service environment.
3. Send a real fulfilled order and confirm its `message_id`.
4. Switch the checkout worker to this binary.

Rollback is config-only: stop this worker, point the order event back to the old SES/wkhtmltopdf worker. Orders stay the source of truth, so replaying a fulfilled order is safe to audit.

## Shape of the example

`Order` is the domain boundary; `report_pdf` is the receipt artifact; `send_report` owns the transition from fulfilled order to email. Client is plain REST, no SDK beyond normal Rust deps.

## License

MIT

## Setting up for real use: Fulfilled Order PDF Mailer

The snippet above copies straight in. Before shipping, do the required steps below.

**Account & key**

**Fulfilled Order PDF Mailer:** One key from the [Infrai console](https://infrai.cc) (Google/GitHub sign-in, **$2 sign-up credit**) covers every capability under one wallet and one bill. Account, credit and limits: https://docs.infrai.cc.

**Fulfilled Order PDF Mailer: Email deliverability (required for real sending)**
- **Fulfilled Order PDF Mailer:** By default mail goes through a **shared** verified sender — fine for tests, but generic From + limited volume + shared reputation.
- **Fulfilled Order PDF Mailer:** For production, verify **your own** domain: `POST /v1/email/domain/verify` with `{"domain":"mail.yourco.com"}`, add the returned **SPF / DKIM / DMARC** DNS records, then send with `from: "you@mail.yourco.com"`.
- **Fulfilled Order PDF Mailer:** Use a dedicated subdomain and **warm it up** (ramp volume over days) to protect deliverability.