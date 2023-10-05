use ethers_core::{
    types::{Bytes, H160, H256, U256, U64},
    utils::{hex, ChainConfig, CliqueConfig, Genesis, GenesisAccount},
};
use hex_literal::hex;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Genesis dev accounts.
pub static GENESIS_DEV_ACCOUNTS: Lazy<Vec<H160>> = Lazy::new(|| {
    vec![
        H160::from_slice(&hex!("3c44cdddb6a900fa2b585dd299e03d12fa4293bc")),
        H160::from_slice(&hex!("70997970c51812dc3a010c7d01b50e0d17dc79c8")),
        H160::from_slice(&hex!("f39fd6e51aad88f6f4ce6ab8827279cfffb92266")),
        H160::from_slice(&hex!("9d14A1992b81dfD355AE83b0b54Dd51582f62db2")),
        H160::from_slice(&hex!("3fab184622dc19b6109349b94811493bf2a45362")),
    ]
});

/// Optimism Mainnet Chain Config.
pub static OPTIMISM_MAINNET_CONFIG: Lazy<ChainConfig> = Lazy::new(|| ChainConfig {
    chain_id: 900,
    homestead_block: Some(0),
    eip150_block: Some(0),
    eip150_hash: Some(H256::zero()),
    eip155_block: Some(0),
    eip158_block: Some(0),
    byzantium_block: Some(0),
    constantinople_block: Some(0),
    petersburg_block: Some(0),
    istanbul_block: Some(0),
    muir_glacier_block: Some(0),
    berlin_block: Some(0),
    london_block: Some(0),
    arrow_glacier_block: Some(0),
    gray_glacier_block: Some(0),
    clique: Some(CliqueConfig {
        period: Some(3),
        epoch: Some(30_000),
    }),
    ..Default::default()
});

/// Genesis Template.
pub static GENESIS_TEMPLATE: Lazy<Genesis> = Lazy::new(|| {
    Genesis {
        config: Default::default(),
        nonce: U64::zero(),
        timestamp: U64::zero(),
        extra_data: Bytes::from(&hex!("0000000000000000000000000000000000000000000000000000000000000000ca062b0fd91172d89bcd4bb084ac4e21972cc4670000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000")),
        gas_limit: U64::from("0xE4E1C0"),
        difficulty: U256::one(),
        mix_hash: H256::zero(),
        coinbase: H160::zero(),
        alloc: Default::default(),
        number: Some(U64::zero()),
        gas_used: Some(U64::zero()),
        parent_hash: Some(H256::zero()),
        base_fee_per_gas: Some(U256::from("0x3B9ACA00")),
    }
});

/// Genesis Allocation Type.
pub type GenesisAlloc = HashMap<H160, GenesisAccount>;

/// Returns the genesis allocations.
pub fn genesis_allocations() -> GenesisAlloc {
    (1..=255)
        .map(|i| {
            (
                H160::from_slice(&hex::decode(format!("{:0>40}", i)).unwrap()),
                GenesisAccount {
                    balance: U256::one(),
                    ..Default::default()
                },
            )
        })
        .collect::<HashMap<_, _>>()
}

/// Returns the developer account allocations.
pub fn dev_accounts() -> GenesisAlloc {
    GENESIS_DEV_ACCOUNTS
        .iter()
        .map(|s| {
            (
                *s,
                GenesisAccount {
                    balance: U256::from(2).pow(U256::from(200)),
                    ..Default::default()
                },
            )
        })
        .collect::<HashMap<_, _>>()
}

/// Returns a genesis template with the given timestamp.
pub fn genesis_template(timestamp: u64) -> Option<Genesis> {
    let mut genesis_allocations = genesis_allocations();
    genesis_allocations.extend(dev_accounts());
    Lazy::<Genesis>::force(&GENESIS_TEMPLATE);
    let genesis = Lazy::get(&GENESIS_TEMPLATE);
    genesis.map(|genesis| {
        let mut genesis = genesis.clone();
        genesis.timestamp = U64::from(timestamp);
        genesis.alloc = genesis_allocations;
        genesis
    })
}

/// Returns the current timestamp in seconds.
pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Returns the genesis template as a string for the given timestamp.
pub fn genesis_template_string(timestamp: u64) -> Option<String> {
    genesis_template(timestamp).map(|genesis| serde_json::to_string_pretty(&genesis).unwrap())
}
