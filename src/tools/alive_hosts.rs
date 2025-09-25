use std::path::PathBuf;

use log::{error, info, warn};
use rusty_falcon::{
    apis::{
        hosts_api::{get_device_details_v2, query_devices_by_filter},
    },
    easy::client::FalconHandle,
    models::{MsaspecPeriodError},
};

// Take in a list of hostnames and find them in crowdstrike, return ones that are not found in crowdstrike.
pub async fn alive_hosts(
    falcon: &FalconHandle,
    hosts: PathBuf,
) -> Result<(), Vec<MsaspecPeriodError>> {
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

    let host_ids =
        query_devices_by_filter(&falcon.cfg, None, None, None, Some(filter.as_str())).await.map_err(|e| {
            error!("Failed to query devices by filter: {}", e);
            vec![MsaspecPeriodError {
                code: 500,
                id: None,
                message: e.to_string(),
            }]
        })?;

    let hosts = get_device_details_v2(&falcon.cfg, host_ids.resources).await;

    info!("{:?}", hosts);
    

    Ok(())
}
