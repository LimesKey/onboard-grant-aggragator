// This program creates a Prometheus exporter with a single metric that tracks
// the number of directories in the specified projects folder.

use env_logger::{Builder, Env};
use log::info;
use prometheus_exporter::prometheus::{register, register_gauge, register_gauge_with_registry, register_histogram, register_int_counter_vec, register_int_gauge, IntCounterVec, Opts, Registry};
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION},
    Client, Url,
};
use std::{collections::HashMap, env, sync::{Arc, Mutex}};
use std::fs;
use std::net::SocketAddr;

mod lib;
use lib::*;

#[tokio::main]
async fn main() {
    // Set up logger with default level info so we can see the messages from
    // prometheus_exporter.
    Builder::from_env(Env::default().default_filter_or("info")).init();

    let addr_raw = "0.0.0.0:8521";
    let addr: SocketAddr = addr_raw.parse().expect("Cannot parse listen address");
    let airtable_api: Result<String, env::VarError> = env::var("AIRTABLE_API");
    let raw_github_api_key: Option<String> = env::var("GITHUB_API").ok();

    let opts = Opts::new("reviewer_pr_count", "Number of pull requests reviewed by each reviewer");
    let counter_vec = register_int_counter_vec!(opts, &["reviewer"]).expect("Failed to create counter vector");
    
    // let reviewer_pr_count = register_gauge!(
    // "reviewer_pr_count", "Number of pull requests reviewed by each reviewer").expect("Failed to create gauge");
    
    let submitted_projects = register_gauge!(
        "submitted_projects",
        "Number of folders in the projects directory in the OnBoard Github"
    )
    .expect("Cannot create gauge onboard_grants_given");

    let transfers_count = register_int_gauge!(
        "transfers_count",
        "Grant transfers out of the OnBoard Hack Club Bank"
    )
    .expect("Cannot create gauge transfers_count");

    // Create the metric
    let average_grant_value = register_gauge!("avg_grant", "Average dollars given per grant")
        .expect("Cannot create gauge average_grant_value");

    let airtable_records_approved_metric =
        register_int_gauge!("airtable_records", "Number of Approved Airtable Records")
            .expect("Cannot create gauge airtable_records_approved_metric");

    let airtable_records_pending_metric = register_int_gauge!(
        "airtable_records_pending",
        "Number of Pending Airtable Records"
    )
    .expect("Cannot create gauge airtable_records_pending_metric");

    let exporter = prometheus_exporter::start(addr).expect("Cannot start exporter");
    loop {
        let _guard = exporter.wait_request();

        info!("Updating metrics");

        submitted_projects.set(count_dirs());
        info!("New directory count: {:?}", submitted_projects);

        airtable_records_approved_metric.set(
            airtable_verifications(airtable_api.clone(), AirTableViews::Approved)
                .await
                .into(),
        );
        info!(
            "New airtable records approved count: {:?}",
            airtable_records_approved_metric
        );

        airtable_records_pending_metric.set(
            airtable_verifications(airtable_api.clone(), AirTableViews::Pending)
                .await
                .into(),
        );
        info!(
            "New airtable records pending count: {:?}",
            airtable_records_pending_metric
        );
        for (reviewer, count) in fetch_pull_requests(raw_github_api_key.clone()).await {
            counter_vec.with_label_values(&[&reviewer]).inc_by(count.into());
        }

        transfers_count.set(count_transfers(&hcb_data().await).into());
        info!("New transfer count: {:?}", transfers_count);

        average_grant_value.set(avg_grant(&hcb_data().await));
        info!("New average grant value: {:?}", average_grant_value);
    }
}

fn count_dirs() -> f64 {
    let temp_projects_path = "projects/";
    // Download the repo and set up the projects directory
    git_download::repo("https://github.com/hackclub/OnBoard")
        .branch_name("main")
        .add_file("projects/", temp_projects_path)
        .exec()
        .unwrap();

    // Read the entries in the projects directory
    let entries = fs::read_dir(temp_projects_path).expect("Failed to read projects directory");

    // Filter and count the directories
    let dir_count = entries
        .filter_map(Result::ok)
        .filter(|e| e.path().is_dir())
        .count() as f64; // Convert to f64, as set() expects a f64

    // Clean up the projects directory
    if fs::remove_dir_all(temp_projects_path).is_ok() {
        info!("Successfully deleted everything in the /projects folder.");
    } else {
        info!("Failed to delete the contents of the /projects folder.");
    }

    dir_count
}

async fn hcb_data() -> Result<Vec<Transfer>, reqwest::Error> {
    let mut page_offset = 0;
    let mut transfers: Vec<Transfer> = Vec::new();

    loop {
        let mut request_url: Url =
            Url::parse("https://hcb.hackclub.com/api/v3/organizations/onboard/transfers/").unwrap();
        request_url.query_pairs_mut().append_pair("per_page", "100");
        request_url
            .query_pairs_mut()
            .append_pair("expand", "transaction");
        request_url
            .query_pairs_mut()
            .append_pair("page", &page_offset.to_string());

        let response = reqwest::get(request_url.as_str()).await?;
        let json = response.json::<serde_json::Value>().await?;
        println!(
            r##"Fetching transfers from page {} from Onboard's Hack Club Bank API using, "{}""##,
            page_offset + 1,
            request_url
        );

        if json.to_string() == "[]" {
            break;
        }

        if let Some(raw_transfers) = json.as_array() {
            for raw_transfer in raw_transfers {
                let transfer = serde_json::from_value(raw_transfer.clone()).unwrap();
                transfers.push(transfer);
            }
        } else {
            println!("Failed to parse JSON array from response");
        }
        page_offset += 1;
    }

    transfers.retain(|transfer| (transfer.amount_cents / 100) <= 100);
    Ok(transfers)
}

fn count_transfers(transfers: &Result<Vec<Transfer>, reqwest::Error>) -> u16 {
    match transfers {
        Ok(count) => return count.len() as u16,
        Err(e) => {
            println!("Failed to fetch transfers: {}", e);
            return 0;
        }
    };
}

fn avg_grant(transfers: &Result<Vec<Transfer>, reqwest::Error>) -> f64 {
    match transfers {
        Ok(transfers) => {
            let mut total = 0;
            for transfer in transfers {
                total += transfer.amount_cents / 100;
            }
            return total as f64 / transfers.len() as f64;
        }
        Err(e) => {
            println!("Failed to fetch transfers: {}", e);
            return 0.0;
        }
    };
}

async fn airtable_verifications(
    api_key: Result<String, env::VarError>,
    AirTableView: AirTableViews,
) -> u16 {
    let max_records = 5000;
    let mut page_offset: Option<String> = None;
    let view;
    match AirTableView {
        AirTableViews::Pending => view = "Pending",
        AirTableViews::Approved => view = "Approved",
    }
    let mut num_records = 0;
    let true_api_key;
    let mut page_offset_count = 0;

    match api_key {
        Ok(key) => {
            info!("Airtable API key found");
            true_api_key = key;
        }
        Err(_) => {
            info!("Airtable API key not found");
            return 0;
        }
    }
    loop {
        let mut request_url: Url =
            Url::parse("https://api.airtable.com/v0/app4Bs8Tjwvk5qcD4/Verifications").unwrap();
        request_url
            .query_pairs_mut()
            .append_pair("maxRecords", &max_records.to_string());
        request_url.query_pairs_mut().append_pair("view", &view);

        match &page_offset {
            Some(offset) => {
                request_url
                    .query_pairs_mut()
                    .append_pair("offset", offset.as_str());
            }
            None => {}
        }

        let auth_token: String = format!("Bearer {}", true_api_key);

        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&auth_token).expect("Invalid header value"),
        );

        let response = Client::new()
            .get(request_url.as_str())
            .headers(headers)
            .send()
            .await;
        let json = response.unwrap().json::<serde_json::Value>().await;
        println!(
            r##"Fetching transfers from OnBoard's AirTable accepted verision forms using, "{}", on page {}."##,
            request_url,
            page_offset_count + 1
        );

        let raw_data = json.unwrap().clone();

        if let Some(records) = raw_data.get("records") {
            if let Some(records_array) = records.as_array() {
                num_records += records_array.len();

                if raw_data.get("offset").is_some() {
                    page_offset = Some(
                        raw_data
                            .get("offset")
                            .unwrap()
                            .to_string()
                            .replace("\"", ""),
                    );
                    page_offset_count += 1;
                } else if page_offset_count > 0 {
                    println!(
                        "[{}]Multiple pages of AirTable data fetched",
                        page_offset_count + 1
                    );
                    return num_records as u16;
                } else {
                    return num_records as u16;
                }

                raw_data.get("error").map(|error| {
                    println!("Error: {}", error);
                });
            } else {
                println!("The AirTable JSON is Invalid");
            }
        } else {
            println!("The AirTable JSON is Invalid : The JSON does not contain a 'records' key");
            return num_records as u16;
        }
    }
}

async fn fetch_pull_requests(github_api_key: Option<String>) -> HashMap<String, u32> {
    let mut page_num = 0;
    let mut reviewer_counts = HashMap::new();
    let mut headers = HeaderMap::new();

    if let Some(api_key) = &github_api_key {
        let auth_token = format!("Bearer {}", api_key);
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&auth_token).expect("Invalid header value"),
        );
        headers.insert(
            reqwest::header::USER_AGENT,
            HeaderValue::from_static("Prometheus Exporter"),
        );
        
        println!("GitHub API key found");
    }
    else {
        println!("No GitHub API key found");
    }

    let client = reqwest::Client::new();

    loop {
        let mut url: Url = Url::parse("https://api.github.com/repos/hackclub/onboard/pulls").unwrap();
        url.query_pairs_mut().append_pair("state", "all");
        url.query_pairs_mut().append_pair("per_page", "100");
        url.query_pairs_mut().append_pair("page", &page_num.to_string());

        println!("Fetching pull requests from {}", url);

        let response = client.get(url.as_str()).headers(headers.clone()).send().await.unwrap().error_for_status().expect("No Response or GitHub API Error");

        let json = response.json::<serde_json::Value>().await.unwrap();
        
        if json.as_array().map_or(false, |arr| arr.is_empty()) {
            return reviewer_counts;
        }

        let pull_requests: Vec<PullRequest> = serde_json::from_value(json).unwrap();
        println!("Number of fetched pull requests {}.", pull_requests.len());

        for pr in pull_requests {
            for reviewer in pr.assignees {
                *reviewer_counts.entry(reviewer.login).or_insert(0) += 1;
            }
        }
        page_num += 1;
    }
}