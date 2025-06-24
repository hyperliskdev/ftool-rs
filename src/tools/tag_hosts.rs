use rusty_falcon::{
    apis::{discover_api::query_hosts, hosts_api::update_device_tags},
    easy::client::FalconHandle,
    models::DeviceapiPeriodUpdateDeviceTagsRequestV1,
};

pub async fn tag_hosts(
    falcon: &FalconHandle,
    tag: String,
    hosts: Option<String>,
    action: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let hostnames = if let Some(file) = hosts {
        std::fs::read_to_string(file)
            .expect("Could not read the file")
            .lines()
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
    } else {
        vec![]
    };

    let host_ids = query_hosts(
        &falcon.cfg,
        None,
        None,
        None,
        Some(format!("hostname:'{}'", hostnames.join("' OR hostname:'")).as_str()),
    )
    .await
    .expect("Failed to query hosts")
    .resources;

    let response = update_device_tags(
        &falcon.cfg,
        DeviceapiPeriodUpdateDeviceTagsRequestV1 {
            action: action.clone(),
            device_ids: host_ids,
            tags: vec![tag.clone()],
        },
    )
    .await
    .expect("Failed to update device tags");

    if response.errors.is_none() {
        println!(
            "Successfully updated tags for hosts: {:?}",
            &response.resources 
        );
        Ok(())
    } else {
        eprintln!("Errors occurred while updating tags: {:?}", &response.errors);
        Err(Box::from("Failed to update device tags"))
    }
}
