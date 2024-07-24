// This program creates a Prometheus exporter with a single metric that tracks
// the number of directories in the specified projects folder.

use env_logger::{Builder, Env};
use log::info;
use prometheus_exporter::prometheus::register_gauge;
use std::fs;
use std::net::SocketAddr;

fn main() {
    // Set up logger with default level info so we can see the messages from
    // prometheus_exporter.
    Builder::from_env(Env::default().default_filter_or("info")).init();
    let temp_projects_path = "projects/";

    // Parse the address used to bind the exporter.
    let addr_raw = "0.0.0.0:8521";
    let addr: SocketAddr = addr_raw.parse().expect("Cannot parse listen address");

    // Create the metric
    let metric = register_gauge!(
        "onboard_grants_given",
        "Number of folders in the projects directory"
    )
    .expect("Cannot create gauge onboard_grants_given");

    // Start the exporter
    let exporter = prometheus_exporter::start(addr).expect("Cannot start exporter");
    let mut dir_count = count_dirs(temp_projects_path);
    loop {
        // Wait for a new request to come in
        let _guard = exporter.wait_request();
        info!("Updating metrics");

        // Update the metric with the current directory count
        metric.set(dir_count);
        dir_count = count_dirs(temp_projects_path);
        info!("New directory count: {}", dir_count);
    }
}

/// Counts the number of directories in the specified path.
///
/// # Arguments
///
/// * `temp_projects_path` - A string slice that holds the path to the projects directory
///
/// # Returns
///
/// * `f64` - The number of directories in the projects directory as a float
fn count_dirs(temp_projects_path: &str) -> f64 {
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