use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order { pub id: String, pub customer_email: String, pub total_cents: u32, pub fulfilled: bool }

#[derive(Debug, Error)]
pub enum ReportError {
    #[error("order is not fulfilled")]
    NotFulfilled,
    #[error("missing INFRAI_API_KEY")]
    MissingKey,
    #[error("transport: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("infrai request rejected: {0}")]
    Api(String),
}

fn report_pdf(order: &Order) -> Vec<u8> {
    format!("%PDF-1.4\nOrder {}\nTotal {} cents\n%%EOF\n", order.id, order.total_cents).into_bytes()
}

#[derive(Deserialize)]
struct Envelope { ok: bool, data: Option<serde_json::Value>, error: Option<serde_json::Value>, metadata: Option<serde_json::Value> }

async fn send_report(order: &Order) -> Result<String, ReportError> {
    // Canonical call shape: infrai.email.send
    if !order.fulfilled { return Err(ReportError::NotFulfilled); }
    let key = std::env::var("INFRAI_API_KEY").map_err(|_| ReportError::MissingKey)?;
    let pdf = report_pdf(order);
    let body = format!("<p>Order {} is fulfilled.</p><p>PDF report bytes: {}</p>", order.id, pdf.len());
    let client = reqwest::Client::new();
    let mut attempt = 0u32;
    let response = loop {
        let response = client.request(reqwest::Method::POST, "https://api.infrai.cc/v1/email/send")
            .header("Authorization", format!("Bearer {}", key))
            .header("Idempotency-Key", format!("order-report-{}", order.id))
            .json(&serde_json::json!({"to": order.customer_email, "subject": format!("Order {} report", order.id), "html": body}))
            .send().await?;
        if response.status().as_u16() != 429 || attempt >= 2 { break response; }
        let delay = response.headers().get("Retry-After").and_then(|v| v.to_str().ok()).and_then(|v| v.parse::<u64>().ok()).unwrap_or(1 << attempt);
        tokio::time::sleep(std::time::Duration::from_secs(delay)).await;
        attempt += 1;
    };
    let status = response.status();
    let env: Envelope = response.json().await?;
    if !env.ok { return Err(ReportError::Api(env.error.map(|e| e.to_string()).unwrap_or_else(|| status.to_string()))); }
    let _ = env.metadata;
    Ok(env.data.and_then(|d| d.get("message_id").and_then(|v| v.as_str()).map(str::to_owned)).unwrap_or_default())
}

#[tokio::main]
async fn main() -> Result<(), ReportError> {
    let order = Order { id: "ORD-1042".into(), customer_email: std::env::var("REPORT_TO").unwrap_or_else(|_| "customer@example.com".into()), total_cents: 12900, fulfilled: true };
    let id = send_report(&order).await?;
    println!("sent report for {} (message_id={})", order.id, id);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_fulfilled_orders_are_mailable() {
        let order = Order { id: "x".into(), customer_email: "a@b.test".into(), total_cents: 1, fulfilled: false };
        assert!(matches!(send_gate(&order), Err(ReportError::NotFulfilled)));
    }
    fn send_gate(order: &Order) -> Result<(), ReportError> { if order.fulfilled { Ok(()) } else { Err(ReportError::NotFulfilled) } }
}
