// This program creates a Prometheus exporter with a single metric that tracks
// the number of directories in the specified projects folder.

use env_logger::{Builder, Env};
use log::info;
use prometheus_exporter::prometheus::register_gauge;
use reqwest::Url;
use std::fs;
use std::net::SocketAddr;
use serde::{Deserialize};
use serde_json::Value;


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

    // Create the metric
    let metric = register_gauge!(
        "onboard_submitted_projects",
        "Number of folders in the projects directory in the OnBoard Github"
    )
    .expect("Cannot create gauge onboard_grants_given");
    hcb_transfers().await;
    // Start the exporter
    let exporter = prometheus_exporter::start(addr).expect("Cannot start exporter");
    let mut dir_count = count_dirs();
    loop {
        // Wait for a new request to come in
        let _guard = exporter.wait_request();
        info!("Updating metrics");

        // Update the metric with the current directory count
        metric.set(dir_count);
        info!("New directory count: {}", dir_count);
        dir_count = count_dirs();
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

async fn hcb_transfers() -> Result<(), reqwest::Error> {
    let page_offset = 10;
    let mut transactions: Vec<Transfer>;

    loop {
        let mut request_url: Url = Url::parse("https://hcb.hackclub.com/api/v3/organizations/onboard/transfers/").unwrap();
        request_url.query_pairs_mut().append_pair("per_page", "100");
        request_url.query_pairs_mut().append_pair("expand", "transaction");
        request_url.query_pairs_mut().append_pair("page", &page_offset.to_string());


        let response = reqwest::get(request_url.as_str()).await?;
        let json: Value = response.json().await?;

        println!(r##"Fetching page {} transfers from Onboard's Hack Club Bank API using, "{}""##, page_offset, request_url);

        if json.is_array() && json.as_array().unwrap().is_empty() {
            break;
        }
    }

    todo!();
    
    //let transfers: Vec<Transfer> = response.json::<Vec<Transfer>>().await?;
    Ok(())
}