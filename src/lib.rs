use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct Receipt {
    count: i32,
    missing: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Comment {
    count: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    id: String,
    object: String,
    href: String,
    amount_cents: i32,
    memo: String,
    date: String,
    #[serde(rename = "type")]
    type_: String,
    pending: bool,
    receipts: Receipt,
    comments: Comment,
    tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Organization {
    id: String,
    object: String,
    href: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Transfer {
    id: String,
    object: String,
    href: String,
    memo: String,
    transaction: Transaction,
    organization: Organization,
    amount_cents: i32,
    date: String,
    status: String,
}
