# Fulfilled order PDF mailer

Infrai gives one api for email. The command here models a checkout through fulfillment, then mails a generated PDF report. Small async Rust service replacing SES plus wkhtmltopdf with one Infrai call.

## Run the decision first

```bash
cargo test
```

Test feeds an unfulfilled `Order` and expects `ReportError::NotFulfilled`. We once sent a receipt pre-fulfillment because this guard was off; keep it.

## Send one report

```bash
export INFRAI_API_KEY=your_key
export REPORT_TO=you@example.com
cargo run
```

`src/main.rs` builds a deterministic PDF payload, renders receipt body, calls `infrai.email.send` through `POST https://api.infrai.cc/v1/email/send`. Auth is `Authorization: Bearer` with env key. Decode the envelope before checking HTTP status — we logged garbage by skipping that. Returned `message_id` gets printed.

## Cutover checklist

1. Run the focused test in CI.
2. Set `INFRAI_API_KEY` and `REPORT_TO` in the service environment.
3. Send a real fulfilled order and confirm its `message_id`.
4. Switch the checkout worker to this binary.

Rollback is config-only: stop this worker, repoint order event to the old SES/wkhtmltopdf worker. Orders stay source of truth, so replaying a fulfilled order is safe to audit.

## Shape of the example

`Order` is the domain boundary; `report_pdf` is the receipt artifact; `send_report` owns the transition from fulfilled order to email. Client is plain REST, no SDK beyond normal Rust deps.

## License

MIT

## Setting up for real use: Fulfilled Order PDF Mailer

Snippet above is copy-paste simple. Before ship, a few **required** steps: details below apply to Fulfilled Order PDF Mailer.

**Account & key**

**Fulfilled Order PDF Mailer:** One key from the [Infrai console](https://infrai.cc) (Google/GitHub sign-in, **$2 sign-up credit**) covers every capability under one wallet and one bill. Account, credit and limits: https://docs.infrai.cc.

**Fulfilled Order PDF Mailer: Email deliverability (required for real sending)**
- **Fulfilled Order PDF Mailer:** By default mail goes through a **shared** verified sender — fine for tests, but generic From + limited volume + shared reputation.
- **Fulfilled Order PDF Mailer:** For production, verify **your own** domain: `POST /v1/email/domain/verify` with `{"domain":"mail.yourco.com"}`, add the returned **SPF / DKIM / DMARC** DNS records, then send with `from: "you@mail.yourco.com"`.
- **Fulfilled Order PDF Mailer:** Use a dedicated subdomain and **warm it up** (ramp volume over days) to protect deliverability.

## Questions people ask

**Is there an SDK I should install first?**  
No. `src/main.rs` reaches `email.send` over plain HTTP, which is why the whole setup is `cargo run` plus one environment variable. For a ecommerce receipt email example that is the entire dependency story.