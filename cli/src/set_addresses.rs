use std::{fs, path::Path};

use eyre::Result;
use serde_json::{Map, Value};

use crate::{constants, utils};

pub fn set_addresses(deployment_dir: &Path) -> Result<(Value, Value)> {
    let mut addresses = Map::new();
    let mut sdk_addresses = Map::new();

    for entry in fs::read_dir(deployment_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().unwrap_or_default() == "json" {
            // We can safely unwrap because the file exists and it has a name
            let file_name = path.file_stem().unwrap().to_string_lossy().to_string();
            let data = utils::read_json(&path)?;

            if let Some(address) = data["address"].as_str() {
                addresses.insert(file_name, address.to_owned().into());
            }
        }
    }

    sdk_addresses.insert("AddressManager".to_owned(), constants::ADDRESS_ZERO.into());
    sdk_addresses.insert("BondManager".to_owned(), constants::ADDRESS_ZERO.into());
    sdk_addresses.insert(
        "StateCommitmentChain".to_owned(),
        constants::ADDRESS_ZERO.into(),
    );
    sdk_addresses.insert(
        "CanonicalTransactionChain".to_owned(),
        constants::ADDRESS_ZERO.into(),
    );
    sdk_addresses.insert(
        "L1CrossDomainMessenger".to_owned(),
        addresses
            .get("Proxy__OVM_L1CrossDomainMessenger")
            .expect("Failed to get L1CrossDomainMessenger address")
            .clone(),
    );
    sdk_addresses.insert(
        "L1StandardBridge".to_owned(),
        addresses
            .get("Proxy__OVM_L1StandardBridge")
            .expect("Failed to get L1StandardBridge address")
            .clone(),
    );
    sdk_addresses.insert(
        "OptimismPortal".to_owned(),
        addresses
            .get("OptimismPortalProxy")
            .expect("Failed to get OptimismPortal address")
            .clone(),
    );
    sdk_addresses.insert(
        "L2OutputOracle".to_owned(),
        addresses
            .get("L2OutputOracleProxy")
            .expect("Failed to get L2OutputOracle address")
            .clone(),
    );

    Ok((Value::Object(addresses), Value::Object(sdk_addresses)))
}
