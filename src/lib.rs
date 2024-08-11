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

#[derive(Deserialize, Serialize, Debug)]
pub struct Transfer {
    id: String,
    object: String,
    href: String,
    transaction: Transaction,
    organization: Organization,
    pub amount_cents: u64,
    date: String,
    status: String,
}

pub enum AirTableViews {
    Pending,
    Approved,
}

#[derive(Deserialize)]
pub struct PullRequest {
    pub number: u32,
    pub assignees: Vec<Assignees>,
    pub labels: Vec<Label>,
}

#[derive(Deserialize)]
pub struct Assignees {
    pub login: String,
}

#[derive(Deserialize)]
pub struct Label {
    pub name: String,
}
