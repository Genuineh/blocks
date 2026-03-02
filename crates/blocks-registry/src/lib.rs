use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use blocks_contract::{BlockContract, ContractLoadError};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct RegisteredBlock {
    pub contract: BlockContract,
    pub contract_path: PathBuf,
}

#[derive(Debug, Default)]
pub struct Registry {
    blocks: BTreeMap<String, RegisteredBlock>,
}

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("blocks root does not exist: {0}")]
    MissingRoot(PathBuf),
    #[error("failed to read blocks root {path}: {source}")]
    ReadRoot {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to read contract {path}: {source}")]
    ReadContract {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse contract {path}: {source}")]
    ParseContract {
        path: PathBuf,
        #[source]
        source: ContractLoadError,
    },
    #[error("duplicate block id: {0}")]
    DuplicateBlockId(String),
}

impl Registry {
    pub fn load_from_root(root: impl AsRef<Path>) -> Result<Self, RegistryError> {
        let root = root.as_ref();
        if !root.exists() {
            return Err(RegistryError::MissingRoot(root.to_path_buf()));
        }

        let entries = fs::read_dir(root).map_err(|source| RegistryError::ReadRoot {
            path: root.to_path_buf(),
            source,
        })?;

        let mut blocks = BTreeMap::new();

        for entry in entries {
            let entry = entry.map_err(|source| RegistryError::ReadRoot {
                path: root.to_path_buf(),
                source,
            })?;
            let block_dir = entry.path();
            if !block_dir.is_dir() {
                continue;
            }

            let contract_path = block_dir.join("block.yaml");
            if !contract_path.is_file() {
                continue;
            }

            let source = fs::read_to_string(&contract_path).map_err(|source| {
                RegistryError::ReadContract {
                    path: contract_path.clone(),
                    source,
                }
            })?;

            let contract = BlockContract::from_yaml_str(&source).map_err(|source| {
                RegistryError::ParseContract {
                    path: contract_path.clone(),
                    source,
                }
            })?;

            if blocks.contains_key(&contract.id) {
                return Err(RegistryError::DuplicateBlockId(contract.id));
            }

            blocks.insert(
                contract.id.clone(),
                RegisteredBlock {
                    contract,
                    contract_path,
                },
            );
        }

        Ok(Self { blocks })
    }

    pub fn list(&self) -> Vec<&RegisteredBlock> {
        self.blocks.values().collect()
    }

    pub fn get(&self, block_id: &str) -> Option<&RegisteredBlock> {
        self.blocks.get(block_id)
    }

    pub fn search(&self, query: &str) -> Vec<&RegisteredBlock> {
        let needle = query.to_ascii_lowercase();

        self.blocks
            .values()
            .filter(|item| {
                item.contract.id.to_ascii_lowercase().contains(&needle)
                    || item
                        .contract
                        .name
                        .as_deref()
                        .unwrap_or_default()
                        .to_ascii_lowercase()
                        .contains(&needle)
            })
            .collect()
    }
}
