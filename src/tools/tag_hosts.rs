use std::{path::{PathBuf}};

use rusty_falcon::{
    apis::{
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
) -> Result<String, Box<dyn std::error::Error>> {
    // Read the file containing hostname.
    // The file is expected to contain one hostname per line.
    let hostnames: Vec<String> = std::fs::read_to_string(&hosts)?
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();

    let filter = format!("hostname:[{}]", hostnames.join(","));
    println!("Filter: {}", filter);
    let host_ids =
        query_devices_by_filter(&falcon.cfg, None, None, None, Some(filter.as_str()))
            .await?;

    if !host_ids.errors.is_empty() {
        return Err(format!("Error querying devices by filter: {:?}", host_ids.errors).into());
    }

    let update_tag = update_device_tags(
        &falcon.cfg,
        DeviceapiPeriodUpdateDeviceTagsRequestV1 {
            action,
            device_ids: host_ids.resources,
            tags: tag,
        },
    )
    .await?;

    if let Some(errors) = &update_tag.errors {
        return Err(format!("Error updating device tags: {:?}", errors).into());
    }


    Ok(
        format!("Successfully updated tags for hosts: {:?}", update_tag.resources).into()
    )
}
