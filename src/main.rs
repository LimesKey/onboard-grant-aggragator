// This program creates a Prometheus exporter with a single metric that tracks
// the number of directories in the specified projects folder.

use env_logger::{Builder, Env};
use log::info;
use prometheus_exporter::prometheus::{register_gauge, register_int_gauge};
use reqwest::Url;
use std::fs;
use std::net::SocketAddr;
use std::env;

mod lib;
use lib::*;

#[tokio::main]
async fn main() {
    // Set up logger with default level info so we can see the messages from
    // prometheus_exporter.
    Builder::from_env(Env::default().default_filter_or("info")).init();
    // Parse the address used to bind the exporter.
    let addr_raw = "0.0.0.0:8521";
    let addr: SocketAddr = addr_raw.parse().expect("Cannot parse listen address");
    let transfer_data = hcb_data().await;
    let airtable_api: Result<String, env::VarError> = env::var("AIRTABLE_API");
    
    match airtable_api {
        Ok(key) => {
            info!("Airtable API key found");
            airtable_verifications(&key);
        }
        Err(_) => {
            info!("Airtable API key not found");
        }
    }

    // Create the metric
    let submitted_projects = register_gauge!(
        "submitted_projects",
        "Number of folders in the projects directory in the OnBoard Github"
    )
    .expect("Cannot create gauge onboard_grants_given");

    let transfers_count = register_int_gauge!(
        "transfers_count",
        "Grant transfers out of the OnBoard Hack Club Bank"
    )
    .expect("Cannot create gauge onboard_grants_given");

    // Create the metric
    let average_grant_value = register_gauge!("avg_grant", "Average dollars given per grant")
        .expect("Cannot create gauge onboard_grants_given");

    // Start the exporter
    let exporter = prometheus_exporter::start(addr).expect("Cannot sta rt exporter");
    loop {
        // Wait for a new request to come in
        let _guard = exporter.wait_request();

        info!("Updating metrics");

        // Update the metric with the current directory count
        submitted_projects.set(count_dirs());
        info!("New directory count: {:?}", submitted_projects);
        transfers_count.set(count_transfers(&transfer_data).into());
        info!("New transfer count: {:?}", transfers_count);
        average_grant_value.set(avg_grant(&transfer_data));
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

async fn airtable_verifications(api_key: &String) -> u16 {
    let max_records = 100;
    let view = String::from("Accepted");
    let mut request_url: Url =
            Url::parse("https://api.airtable.com/v0/app4Bs8Tjwvk5qcD4/Verifications").unwrap();
        request_url.query_pairs_mut().append_pair("maxRecords", &max_records.to_string());
        request_url
            .query_pairs_mut()
            .append_pair("view", &view);

    let response = reqwest::get(request_url.as_str()).await;
    let json = response.unwrap().json::<serde_json::Value>().await;
    println!(
            r##"Fetching transfers from OnBoard's AirTable accepted verision forms using, "{}""##, request_url);
    println!("JSON {:?}", json);
    15
}