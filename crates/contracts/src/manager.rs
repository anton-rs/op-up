use std::{
    fs,
    path::{Path, PathBuf},
};

use eyre::Result;
use serde_json::{Map, Value};

use ethers_core::types::Address;

/// AddressManager
///
/// The address manger.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct AddressManager {
    /// The path to the deployment directory.
    pub deployment_dir: PathBuf,
}

impl AddressManager {
    /// Creates a new address manager.
    pub fn new(deployment_dir: PathBuf) -> Self {
        Self { deployment_dir }
    }

    /// Returns the address of the given contract.
    pub fn get_address(&self, contract: &str) -> Result<Address> {
        let file_name = format!("{}.json", contract);
        let path = self.deployment_dir.join(file_name);

        if path.exists() {
            let data = read_json(&path)?;
            let address = data["address"].as_str().unwrap();

            Ok(address.parse()?)
        } else {
            Err(eyre::eyre!("No address found for contract: {}", contract))
        }
    }

    /// Returns a set of addresses and SDK addresses using the given deployment directory.
    pub fn set_addresses(deployment_dir: &Path) -> Result<(Value, Value)> {
        let mut addresses = Map::new();
        let mut sdk_addresses = Map::new();

        for entry in fs::read_dir(deployment_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().unwrap_or_default() == "json" {
                // We can safely unwrap because the file exists and it has a name
                let file_name = path.file_stem().unwrap().to_string_lossy().to_string();
                let data = read_json(&path)?;

                if let Some(address) = data["address"].as_str() {
                    addresses.insert(file_name, address.to_owned().into());
                }
            }
        }

        sdk_addresses.insert(
            "AddressManager".to_owned(),
            Address::zero().to_string().into(),
        );
        sdk_addresses.insert("BondManager".to_owned(), Address::zero().to_string().into());
        sdk_addresses.insert(
            "StateCommitmentChain".to_owned(),
            Address::zero().to_string().into(),
        );
        sdk_addresses.insert(
            "CanonicalTransactionChain".to_owned(),
            Address::zero().to_string().into(),
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
}

/// Read a JSON file and return a `serde_json::Value`.
pub fn read_json(file_path: &Path) -> Result<Value> {
    let file = std::fs::File::open(file_path)?;
    let reader = std::io::BufReader::new(file);
    let json_value: Value = serde_json::from_reader(reader)?;
    Ok(json_value)
}
