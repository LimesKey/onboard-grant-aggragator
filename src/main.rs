// Will create an exporter with a single metric that does not change

use env_logger::{
    Builder,
    Env,
};
use log::info;
use prometheus_exporter::prometheus::register_gauge;
use std::net::SocketAddr;
use std::fs;


fn main() {
    // Setup logger with default level info so we can see the messages from
    // prometheus_exporter.
    Builder::from_env(Env::default().default_filter_or("info")).init();
    let temp_projects_path = "projects/";

    // Parse address used to bind exporter to.
    let addr_raw = "0.0.0.0:8521";
    let addr: SocketAddr = addr_raw.parse().expect("can not parse listen addr");

    // Create metric
    let metric = register_gauge!("onboard_grants_given", "Number of folders in the projects directory")
        .expect("can not create gauge simple_the_answer");

    // Start exporter
    let exporter = prometheus_exporter::start(addr).expect("can not start exporter");

    // // Get metrics from exporter
    // std::thread::spawn(move || {
    //     loop {
    //         std::thread::sleep(std::time::Duration::from_millis(1000));

    //         // Get metrics from exporter
    //         let body = reqwest::blocking::get(format!("http://{addr_raw}/metrics"))
    //             .expect("can not get metrics from exporter")
    //             .text()
    //             .expect("can not body text from request");

    //         info!("Exporter metrics:\n{}", body);
    //     }
    // });

    loop {
        // Will block until a new request comes in.
        let _guard = exporter.wait_request();
        info!("Updating metrics");

        // Update metric with random value.
        let dir_count = count_dirs(temp_projects_path);
        info!("New random value: {}", dir_count);

        metric.set(dir_count);
    }
}

fn count_dirs(temp_projects_path: &str) -> f64 {
    git_download::repo("https://github.com/hackclub/OnBoard")
    .branch_name("main")
    .add_file("projects/", temp_projects_path)
    .exec().unwrap();

    // set the metric value to the amount of directories in the projects folder
    
    let entries = fs::read_dir(temp_projects_path).expect("Failed to read projects directory");

// Filter and count the directories
    let dir_count = entries.filter_map(Result::ok)
    .filter(|e| e.path().is_dir())
    .count() as f64; // Convert to f64, as set() expects a f64

    if fs::remove_dir_all(temp_projects_path).is_ok() {
        println!("Successfully deleted everything in the /projects folder.");
    } else {
        println!("Failed to delete the contents of the /projects folder.");
    }

    dir_count
}