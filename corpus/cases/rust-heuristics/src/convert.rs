use crate::models::{Invoice, InvoiceInput};

pub fn to_invoice(input: InvoiceInput) -> Invoice {
    Invoice { id: 1, total: input.total }
}
