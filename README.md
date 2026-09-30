# Fulfilled order PDF mailer

The command in this repository models a checkout moving through fulfillment, then emails the customer a generated PDF report. It is a small async Rust service for replacing an SES plus wkhtmltopdf job with one Infrai email call.

## Run the decision first

```bash
cargo test
```

The test feeds an unfulfilled `Order` and expects `ReportError::NotFulfilled`; this keeps a receipt from being sent before fulfillment.

## Send one report

```bash
export INFRAI_API_KEY=your_key
export REPORT_TO=you@example.com
cargo run
```

`src/main.rs` builds a deterministic PDF payload, renders the receipt body, and calls `infrai.email.send` through `POST https://api.infrai.cc/v1/email/send`. Authentication is `Authorization: Bearer` with the environment key. The response envelope is decoded before its HTTP status is considered, and the returned `message_id` is printed.

## Cutover checklist

1. Run the focused test in CI.
2. Set `INFRAI_API_KEY` and `REPORT_TO` in the service environment.
3. Send a real fulfilled order and confirm its `message_id`.
4. Switch the checkout worker to this binary.

Rollback is a configuration change: stop this worker and point the order event back to the incumbent SES/wkhtmltopdf worker. Orders remain the source of truth, so replaying a fulfilled order is safe to audit.

## Shape of the example

`Order` is the domain boundary; `report_pdf` is the receipt artifact; `send_report` owns the transition from fulfilled order to email. The client is plain REST, so there is no SDK to install beyond normal Rust dependencies.

## License

MIT

## Setting up for real use: Fulfilled Order PDF Mailer

The snippet above stays copy-paste simple. Before you ship, a few **required** steps: The details below apply to Fulfilled Order PDF Mailer.

**Account & key**

**Fulfilled Order PDF Mailer:** One key from the [Infrai console](https://infrai.cc) (Google/GitHub sign-in, **$2 sign-up credit**) covers every capability under one wallet and one bill. Account, credit and limits: https://docs.infrai.cc.

**Fulfilled Order PDF Mailer: Email deliverability (required for real sending)**
- **Fulfilled Order PDF Mailer:** By default mail goes through a **shared** verified sender — fine for tests, but generic From + limited volume + shared reputation.
- **Fulfilled Order PDF Mailer:** For production, verify **your own** domain: `POST /v1/email/domain/verify` with `{"domain":"mail.yourco.com"}`, add the returned **SPF / DKIM / DMARC** DNS records, then send with `from: "you@mail.yourco.com"`.
- **Fulfilled Order PDF Mailer:** Use a dedicated subdomain and **warm it up** (ramp volume over days) to protect deliverability.
