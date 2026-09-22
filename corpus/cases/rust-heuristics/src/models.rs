use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Invoice {
    pub id: u64,
    pub total: u64,
}

#[derive(Serialize, Deserialize)]
pub struct InvoiceInput {
    pub total: u64,
}
