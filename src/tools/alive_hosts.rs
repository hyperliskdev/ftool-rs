use std::path::PathBuf;

use log::{error, info, warn};
use rusty_falcon::{
    apis::{discover_api::get_hosts, hosts_api::{get_device_details_v2, query_devices_by_filter}},
    easy::client::FalconHandle,
    models::{DeviceapiPeriodDeviceDetailsResponseSwagger, DomainPeriodDiscoverApiHostEntitiesResponse},
};

// Take in a list of hostnames and find them in crowdstrike, return ones that are not found in crowdstrike.
pub async fn alive_hosts(
    falcon: &FalconHandle,
    hosts: PathBuf,
) -> Result<DomainPeriodDiscoverApiHostEntitiesResponse, Box<dyn std::error::Error>> {
    let hostnames: Vec<String> = std::fs::read_to_string(&hosts)
        .expect("Failed to read hosts file")
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();

    info!("Searching for devices in the hostname file...");

    let filter = format!(
        "hostname:[{}]",
        hostnames
            .iter()
            .map(|h| format!("'{}'", h))
            .collect::<Vec<_>>()
            .join(",")
    );

    info!("Querying devices for sensor_ids.");

    let host_ids = match query_devices_by_filter(&falcon.cfg, None, None, None, Some(filter.as_str())).await {
        Ok(hosts) => hosts,
        Err(e) => {
            error!("Error querying devices: {:?}", e);
            return Err(Box::new(e));
        }
    };

    info!("Found host_ids: {:?}", &host_ids.resources);

    let hosts = get_hosts(&falcon.cfg, host_ids.resources).await?;

    Ok(hosts)
}
