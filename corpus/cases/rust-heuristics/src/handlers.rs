use axum::{routing::post, Json, Router};

use crate::models::{Invoice, InvoiceInput};
use crate::convert;

pub fn routes() -> Router {
    Router::new().route("/invoices", post(create_invoice))
}

async fn create_invoice(Json(input): Json<InvoiceInput>) -> Json<Invoice> {
    Json(convert::to_invoice(input))
}
