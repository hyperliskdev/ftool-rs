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

    let hostnames_count = &hostnames.len();

    let filter = format!(
        "hostname:[{}]",
        hostnames
            .iter()
            .map(|h| format!("'{}'", h))
            .collect::<Vec<_>>()
            .join(",")
    );

    let host_ids =
        query_devices_by_filter(&falcon.cfg, None, None, None, Some(filter.as_str())).await.map_err(|e| {
            error!("Failed to query devices by filter: {}", e);
            vec![MsaspecPeriodError {
                code: 500,
                id: None,
                message: e.to_string(),
            }]
        })?;

    info!("host ids: {:?}", &host_ids);
    
    // Take the host_ids and pear them down in the get_hosts query to be a Vec<String> of hostnames that exist in Crowdstrike.
    let host_ids = host_ids.resources;

    let hosts = get_device_details_v2(&falcon.cfg, host_ids).await;

    // info!("hosts value: {:?}", &hosts.ok().unwrap());

    let mut count = 0;
    for host in hosts.ok().unwrap().resources {
        let hostname = host.hostname.unwrap();

        info!("HOST: {:?}", &hostname);

        if hostnames.contains(&hostname) {
            count = count + 1;
        } else {
            info!("Could not find {:?}", &hostname)
        }
    }

    warn!("Found {:?} devices in crowdstrike out of {:?}.", count, hostnames_count);

    Ok(())
}
