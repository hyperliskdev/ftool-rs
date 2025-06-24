use std::path::{Path, PathBuf};

use rusty_falcon::{
    apis::{
        discover_api::query_hosts,
        hosts_api::{query_devices_by_filter, update_device_tags},
    },
    easy::client::FalconHandle,
    models::DeviceapiPeriodUpdateDeviceTagsRequestV1,
};

pub async fn tag_hosts(
    falcon: &FalconHandle,
    tag: Vec<String>,
    hosts: PathBuf,
    action: String,
) -> Result<(), Box<dyn std::error::Error>> {
    // Read the file containing hostname.
    // The file is expected to contain one hostname per line.
    let hostnames: Vec<String> = std::fs::read_to_string(&hosts)?
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();

    let filter = format!("hostname:[{}]", hostnames.join(","));
    let host_ids =
        query_devices_by_filter(&falcon.cfg, None, None, None, Some(filter.as_str()))
            .await
            .inspect(|response| {
                if !response.errors.is_empty() {
                    eprintln!("Error querying devices by filter: {:?}", response.errors);
                }
            })?;

    let tag_update = update_device_tags(&falcon.cfg, DeviceapiPeriodUpdateDeviceTagsRequestV1 { action: action, device_ids: host_ids.resources, tags: tag })
        .await
        .inspect(|response| {
            if !response.errors.is_empty() {
                eprintln!("Error updating device tags: {:?}", response.errors);
            }
        })?;

    if host_ids.resources.is_empty() {
        eprintln!("No hosts found for the provided hostnames.");
        return Ok(());
    }
}
