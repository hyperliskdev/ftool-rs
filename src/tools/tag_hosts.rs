use std::path::PathBuf;

use log::{error, info};
use rusty_falcon::{
    apis::hosts_api::{query_devices_by_filter, update_device_tags},
    easy::client::FalconHandle,
    models::{DeviceapiPeriodUpdateDeviceDetailsResponseV1, DeviceapiPeriodUpdateDeviceTagsRequestV1, MsaspecPeriodError},
};


pub async fn tag_hosts(
    falcon: &FalconHandle,
    tag: Vec<String>,
    hosts: PathBuf,
    action: String,
) -> Result<Vec<DeviceapiPeriodUpdateDeviceDetailsResponseV1> , Vec<MsaspecPeriodError>> {
    // Read the file containing hostname.
    // The file is expected to contain one hostname per line.
    let hostnames: Vec<String> = std::fs::read_to_string(&hosts).expect("Failed to read hosts file")
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();

    let filter = format!(
        "hostname:[{}]",
        hostnames
            .iter()
            .map(|h| format!("'{}'", h))
            .collect::<Vec<_>>()
            .join(",")
    );
    
    info!("Querying devices with filter: {}", filter);


    let host_ids =
        query_devices_by_filter(&falcon.cfg, None, None, None, Some(filter.as_str())).await.map_err(|e| {
            error!("Failed to query devices by filter: {}", e);
            vec![MsaspecPeriodError {
                code: 500,
                id: None,
                message: e.to_string(),
            }]
        })?;

    if !host_ids.errors.is_empty() {
        return Err(host_ids.errors);
    }

    let update_tag = update_device_tags(
        &falcon.cfg,
        DeviceapiPeriodUpdateDeviceTagsRequestV1 {
            action,
            device_ids: host_ids.resources,
            tags: tag,
        },
    )
    .await.map_err(|e| {
        error!("Failed to update device tags: {}", e);
        vec![MsaspecPeriodError {
            code: 500,
            id: None,
            message: e.to_string(),
        }]
    })?;

    if let Some(ref errors) = update_tag.errors {
        error!("Errors occurred while updating device tags: {:?}", errors);
        return Err(errors.clone());
    }

    Ok(update_tag.resources)
}
