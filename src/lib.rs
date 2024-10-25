use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Comment {
    count: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    id: String,
    object: String,
    href: String,
    amount_cents: u64,
    memo: String,
    date: String,
    #[serde(rename = "type")]
    type_: String,
    pending: bool,
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

#[derive(Deserialize, Clone)]
pub struct PullRequest {
    pub number: u32,
    pub assignees: Vec<Assignees>,
    pub labels: Vec<Label>,
    pub requested_reviewers: Vec<Reviewers>,
    pub state: State,
    pub merged_at: Option<String>,
}

#[derive(Deserialize, Clone)]
pub struct Assignees {
    pub login: String,
}

#[derive(Deserialize, Clone)]
pub struct Reviewers {
    pub login: String,
}

#[derive(Deserialize, Clone)]
pub struct Label {
    pub name: String,
}

pub enum AirTableViews {
    Pending,
    Approved,
}

#[derive(Deserialize, PartialEq, Clone)]
pub enum State {
    open,
    closed,
    merged,
    any,
}
