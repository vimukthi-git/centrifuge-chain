use babe_primitives::AuthorityId as BabeId;
use centrifuge_chain_runtime::{
    AccountId, BabeConfig, BalancesConfig, GenesisConfig, GrandpaConfig, IndicesConfig, SudoConfig,
    SystemConfig, WASM_BINARY,
};
use grandpa_primitives::AuthorityId as GrandpaId;
use primitives::{Pair, Public};
use substrate_service;
use hex::FromHex;

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = substrate_service::ChainSpec<GenesisConfig>;

/// The chain specification option. This is expected to come in from the CLI and
/// is little more than one of a number of alternatives which can easily be converted
/// from a string (`--chain=...`) into a `ChainSpec`.
#[derive(Clone, Debug)]
pub enum Alternative {
    /// Whatever the current runtime is, with just Alice as an auth.
    Development,
    /// Whatever the current runtime is, with simple Alice/Bob auths.
    LocalTestnet,
    /// Fulvous testnet with whatever the current runtime is, with simple Alice/Bob auths and sudo account set by environment.
    Fulvous,
}

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

pub fn get_from_pub_str<TPublic: Public>(pubkey_hex: &str) -> AccountId {
    primitives::sr25519::Public::from_raw(
        byte32_from_slice(Vec::from_hex(pubkey_hex)
            .expect("a static hex string is valid")
            .as_slice()))
}

fn byte32_from_slice(bytes: &[u8]) -> [u8; 32] {
    let mut array = [0; 32];
    let bytes = &bytes[..array.len()];
    array.copy_from_slice(bytes);
    array
}

/// Helper function to generate stash, controller and session key from seed
pub fn get_authority_keys_from_seed(seed: &str) -> (AccountId, AccountId, GrandpaId, BabeId) {
    (
        get_from_seed::<AccountId>(&format!("{}//stash", seed)),
        get_from_seed::<AccountId>(seed),
        get_from_seed::<GrandpaId>(seed),
        get_from_seed::<BabeId>(seed),
    )
}

impl Alternative {
    /// Get an actual chain config from one of the alternatives.
    pub(crate) fn load(self) -> Result<ChainSpec, String> {
        Ok(match self {
            Alternative::Development => ChainSpec::from_genesis(
                "Development",
                "dev",
                || {
                    testnet_genesis(
                        vec![get_authority_keys_from_seed("Alice")],
                        get_from_seed::<AccountId>("Alice"),
                        vec![
                            get_from_seed::<AccountId>("Alice"),
                            get_from_seed::<AccountId>("Bob"),
                            get_from_seed::<AccountId>("Alice//stash"),
                            get_from_seed::<AccountId>("Bob//stash"),
                        ],
                        true,
                    )
                },
                vec![],
                None,
                None,
                None,
                None,
            ),
            Alternative::LocalTestnet => ChainSpec::from_genesis(
                "Local Testnet",
                "local_testnet",
                || testnet_genesis(vec![
                    get_authority_keys_from_seed("Alice"),
                    get_authority_keys_from_seed("Bob"),
                ],
                                   get_from_seed::<AccountId>("Alice"),
                                   vec![
                                       get_from_seed::<AccountId>("Alice"),
                                       get_from_seed::<AccountId>("Bob"),
                                       get_from_seed::<AccountId>("Charlie"),
                                       get_from_seed::<AccountId>("Dave"),
                                       get_from_seed::<AccountId>("Eve"),
                                       get_from_seed::<AccountId>("Ferdie"),
                                       get_from_seed::<AccountId>("Alice//stash"),
                                       get_from_seed::<AccountId>("Bob//stash"),
                                       get_from_seed::<AccountId>("Charlie//stash"),
                                       get_from_seed::<AccountId>("Dave//stash"),
                                       get_from_seed::<AccountId>("Eve//stash"),
                                       get_from_seed::<AccountId>("Ferdie//stash"),
                                   ],
                                   true),
                vec![],
                None,
                None,
                None,
                None
            ),
            // Fulvous initial config
            Alternative::Fulvous => ChainSpec::from_genesis(
                "Fulvous Testnet",
                "fulvous",
                || {
                    testnet_genesis(
                        vec![
                            // TODO remove Alice and Bob here and setup proper validator accounts. Then following RPC methods needs to be called on a validator node to start validating.
                            // curl -H 'Content-Type: application/json' --data '{ "jsonrpc":"2.0", "method":"author_insertKey", "params":["gran", "seed"],"id":1 }' localhost:9933
                            // curl -H 'Content-Type: application/json' --data '{ "jsonrpc":"2.0", "method":"author_insertKey", "params":["babe", "seed"],"id":1 }' localhost:9933
                            get_authority_keys_from_seed("Alice"),
                            get_authority_keys_from_seed("Bob"),
                        ],
                        get_from_pub_str::<AccountId>("c405224448dcd4259816b09cfedbd8df0e6796b16286ea18efa2d6343da5992e"),
                        vec![
                            get_from_pub_str::<AccountId>("c405224448dcd4259816b09cfedbd8df0e6796b16286ea18efa2d6343da5992e")
                        ],
                        true,
                    )
                },
                vec![],
                None,
                Some("centrifuge-chain"),
                None,
                None,
            ),
        })
    }

    pub(crate) fn from(s: &str) -> Option<Self> {
        match s {
            "dev" => Some(Alternative::Development),
            "" | "local" => Some(Alternative::LocalTestnet),
            "fulvous" => Some(Alternative::Fulvous),
            _ => None,
        }
    }
}

fn testnet_genesis(
    initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    _enable_println: bool,
) -> GenesisConfig {
    GenesisConfig {
        system: Some(SystemConfig {
            code: WASM_BINARY.to_vec(),
            changes_trie_config: Default::default(),
        }),
        indices: Some(IndicesConfig {
            ids: endowed_accounts.clone(),
        }),
        balances: Some(BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, 1 << 60))
                .collect(),
            vesting: vec![],
        }),
        sudo: Some(SudoConfig { key: root_key }),
        babe: Some(BabeConfig {
            authorities: initial_authorities
                .iter()
                .map(|x| (x.3.clone(), 1))
                .collect(),
        }),
        grandpa: Some(GrandpaConfig {
            authorities: initial_authorities
                .iter()
                .map(|x| (x.2.clone(), 1))
                .collect(),
        }),
    }
}
