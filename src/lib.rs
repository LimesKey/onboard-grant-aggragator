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

#[derive(Debug, Deserialize)]
pub struct Assignees {
    pub nodes: Vec<User>,
}

#[derive(Debug, Deserialize)]
pub struct User {
    pub login: String,
}

#[derive(Debug, Deserialize)]
pub struct PageInfo {
    pub endCursor: Option<String>,
    pub hasNextPage: bool,
}

#[derive(Debug, Deserialize)]
pub struct GraphQLError {
    pub message: String,
}

#[derive(Serialize)]
pub struct GraphQLQuery {
    pub query: String,
    pub variables: Variables,
}

#[derive(Serialize)]
pub struct Variables {
    pub cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ResponseData {
    pub data: Option<RepositoryData>,
    pub errors: Option<Vec<GraphQLError>>,
}

#[derive(Debug, Deserialize)]
pub struct RepositoryData {
    pub search: Search,
}

#[derive(Debug, Deserialize)]
pub struct Search {
    issueCount: u32,
    pub nodes: Vec<PullRequestNode>,
    pub pageInfo: PageInfo,
}

#[derive(Debug, Deserialize)]
pub struct PullRequestNode {
    number: u32,
    pub mergedBy: Option<User>,
    pub assignees: Assignees,
}
