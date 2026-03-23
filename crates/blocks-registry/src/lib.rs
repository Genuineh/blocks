use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use blocks_contract::{BlockContract, ContractLoadError, ContractValidationIssue};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct RegisteredBlock {
    pub contract: BlockContract,
    pub block_dir: PathBuf,
    pub contract_path: PathBuf,
    pub implementation_path: PathBuf,
    pub contract_warnings: Vec<ContractValidationIssue>,
}

#[derive(Debug, Default)]
pub struct Registry {
    blocks: BTreeMap<String, RegisteredBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageProvider {
    Workspace(PathBuf),
    File(PathBuf),
    Remote(String),
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
    #[error("missing implementation metadata in contract {path}")]
    MissingImplementationMetadata { path: PathBuf },
    #[error("implementation entry for block {block_id} does not exist: {path}")]
    MissingImplementationEntry { block_id: String, path: PathBuf },
    #[error("duplicate block id: {0}")]
    DuplicateBlockId(String),
    #[error("invalid package provider: {0}")]
    InvalidPackageProvider(String),
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

            let (contract, report) =
                BlockContract::from_yaml_str_with_report(&source).map_err(|source| {
                    RegistryError::ParseContract {
                        path: contract_path.clone(),
                        source,
                    }
                })?;
            let implementation = contract.implementation.as_ref().ok_or_else(|| {
                RegistryError::MissingImplementationMetadata {
                    path: contract_path.clone(),
                }
            })?;
            let implementation_path = block_dir.join(&implementation.entry);

            if !implementation_path.is_file() {
                return Err(RegistryError::MissingImplementationEntry {
                    block_id: contract.id.clone(),
                    path: implementation_path,
                });
            }

            if blocks.contains_key(&contract.id) {
                return Err(RegistryError::DuplicateBlockId(contract.id));
            }

            blocks.insert(
                contract.id.clone(),
                RegisteredBlock {
                    contract,
                    block_dir,
                    contract_path,
                    implementation_path,
                    contract_warnings: report.warnings(),
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

impl PackageProvider {
    pub fn parse(value: &str) -> Result<Self, RegistryError> {
        if let Some(path) = value.strip_prefix("workspace:") {
            return Ok(Self::Workspace(PathBuf::from(path)));
        }
        if let Some(path) = value.strip_prefix("file:") {
            return Ok(Self::File(PathBuf::from(path)));
        }
        if let Some(endpoint) = value.strip_prefix("remote:") {
            return Ok(Self::Remote(endpoint.to_string()));
        }
        Err(RegistryError::InvalidPackageProvider(value.to_string()))
    }

    pub fn label(&self) -> String {
        match self {
            Self::Workspace(path) => format!("workspace:{}", path.display()),
            Self::File(path) => format!("file:{}", path.display()),
            Self::Remote(endpoint) => format!("remote:{endpoint}"),
        }
    }
}
