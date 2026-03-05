use std::collections::BTreeSet;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use blocks_contract::{BlockContract, ImplementationKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RegisteredBlock {
    pub(crate) dependency_name: String,
    pub(crate) block_id: String,
    pub(crate) metadata_path: PathBuf,
}

#[derive(Debug)]
pub(crate) enum CodegenError {
    ReadManifest {
        path: PathBuf,
        source: std::io::Error,
    },
    InvalidManifest(String),
    MissingPathDependency {
        dependency_name: String,
    },
    MissingBlockMetadata {
        dependency_name: String,
        path: PathBuf,
    },
    ReadBlockMetadata {
        path: PathBuf,
        source: std::io::Error,
    },
    InvalidBlockMetadata {
        dependency_name: String,
        path: PathBuf,
        message: String,
    },
    DuplicateBlockId {
        block_id: String,
        existing_dependency: String,
        duplicate_dependency: String,
    },
    WriteGeneratedFile {
        path: PathBuf,
        source: std::io::Error,
    },
}

impl fmt::Display for CodegenError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadManifest { path, source } => {
                write!(
                    formatter,
                    "failed to read catalog manifest {}: {source}",
                    path.display()
                )
            }
            Self::InvalidManifest(message) => write!(formatter, "{message}"),
            Self::MissingPathDependency { dependency_name } => write!(
                formatter,
                "catalog dependency {dependency_name} must declare an inline table with a path"
            ),
            Self::MissingBlockMetadata {
                dependency_name,
                path,
            } => write!(
                formatter,
                "catalog dependency {dependency_name} is missing sibling block metadata: {}",
                path.display()
            ),
            Self::ReadBlockMetadata { path, source } => {
                write!(
                    formatter,
                    "failed to read block metadata {}: {source}",
                    path.display()
                )
            }
            Self::InvalidBlockMetadata {
                dependency_name,
                path,
                message,
            } => write!(
                formatter,
                "catalog dependency {dependency_name} has invalid block metadata {}: {message}",
                path.display()
            ),
            Self::DuplicateBlockId {
                block_id,
                existing_dependency,
                duplicate_dependency,
            } => write!(
                formatter,
                "block id {block_id} is registered by both {existing_dependency} and {duplicate_dependency}"
            ),
            Self::WriteGeneratedFile { path, source } => write!(
                formatter,
                "failed to write generated catalog glue {}: {source}",
                path.display()
            ),
        }
    }
}

pub(crate) fn write_generated_catalog(
    manifest_path: &Path,
    output_path: &Path,
) -> Result<Vec<PathBuf>, CodegenError> {
    let blocks = collect_registered_blocks(manifest_path)?;
    let rendered = render_dispatch_glue(&blocks);

    fs::write(output_path, rendered).map_err(|source| CodegenError::WriteGeneratedFile {
        path: output_path.to_path_buf(),
        source,
    })?;

    let mut metadata_paths = Vec::with_capacity(blocks.len());
    for block in blocks {
        metadata_paths.push(block.metadata_path);
    }

    Ok(metadata_paths)
}

pub(crate) fn collect_registered_blocks(
    manifest_path: &Path,
) -> Result<Vec<RegisteredBlock>, CodegenError> {
    let manifest_dir = manifest_path.parent().ok_or_else(|| {
        CodegenError::InvalidManifest(format!(
            "catalog manifest path has no parent directory: {}",
            manifest_path.display()
        ))
    })?;
    let manifest_source =
        fs::read_to_string(manifest_path).map_err(|source| CodegenError::ReadManifest {
            path: manifest_path.to_path_buf(),
            source,
        })?;
    let dependencies = parse_block_dependencies(&manifest_source)?;
    let mut blocks: Vec<RegisteredBlock> = Vec::with_capacity(dependencies.len());
    let mut seen_ids = BTreeSet::new();

    for dependency in dependencies {
        let dependency_dir = resolve_dependency_dir(manifest_dir, &dependency.path);
        let metadata_path = dependency_dir.join("..").join("block.yaml");
        let metadata_path = metadata_path
            .canonicalize()
            .unwrap_or_else(|_| metadata_path.clone());

        if !metadata_path.is_file() {
            return Err(CodegenError::MissingBlockMetadata {
                dependency_name: dependency.name,
                path: metadata_path,
            });
        }

        let metadata_source = fs::read_to_string(&metadata_path).map_err(|source| {
            CodegenError::ReadBlockMetadata {
                path: metadata_path.clone(),
                source,
            }
        })?;
        let contract = BlockContract::from_yaml_str(&metadata_source).map_err(|error| {
            CodegenError::InvalidBlockMetadata {
                dependency_name: dependency.name.clone(),
                path: metadata_path.clone(),
                message: error.to_string(),
            }
        })?;
        let implementation =
            contract
                .implementation
                .ok_or_else(|| CodegenError::InvalidBlockMetadata {
                    dependency_name: dependency.name.clone(),
                    path: metadata_path.clone(),
                    message: "missing implementation metadata".to_string(),
                })?;

        if implementation.kind != ImplementationKind::Rust {
            return Err(CodegenError::InvalidBlockMetadata {
                dependency_name: dependency.name.clone(),
                path: metadata_path.clone(),
                message: "catalog can only register rust blocks".to_string(),
            });
        }

        if !seen_ids.insert(contract.id.clone()) {
            let existing_dependency = blocks
                .iter()
                .find(|block| block.block_id == contract.id)
                .map(|block| block.dependency_name.clone())
                .unwrap_or_else(|| "<unknown>".to_string());
            return Err(CodegenError::DuplicateBlockId {
                block_id: contract.id,
                existing_dependency,
                duplicate_dependency: dependency.name,
            });
        }

        blocks.push(RegisteredBlock {
            dependency_name: dependency.name,
            block_id: contract.id,
            metadata_path,
        });
    }

    blocks.sort_by(|left, right| {
        left.block_id
            .cmp(&right.block_id)
            .then_with(|| left.dependency_name.cmp(&right.dependency_name))
    });

    Ok(blocks)
}

pub(crate) fn render_dispatch_glue(blocks: &[RegisteredBlock]) -> String {
    let mut rendered = String::from("// @generated by build.rs\n");
    rendered.push_str("const REGISTERED_BLOCK_IDS: &[&str] = &[\n");
    for block in blocks {
        rendered.push_str("    \"");
        rendered.push_str(&block.block_id);
        rendered.push_str("\",\n");
    }
    rendered.push_str("];\n\n");
    rendered.push_str(
        "fn dispatch_registered_block(\n    block_id: &str,\n    input: &Value,\n) -> Option<Result<Value, BlockExecutionError>> {\n    match block_id {\n",
    );
    for block in blocks {
        rendered.push_str("        \"");
        rendered.push_str(&block.block_id);
        rendered.push_str("\" => Some(");
        rendered.push_str(&dependency_module_name(&block.dependency_name));
        rendered.push_str("::run(input)),\n");
    }
    rendered.push_str("        _ => None,\n    }\n}\n");
    rendered
}

#[derive(Debug)]
struct ManifestDependency {
    name: String,
    path: PathBuf,
}

fn parse_block_dependencies(source: &str) -> Result<Vec<ManifestDependency>, CodegenError> {
    let mut dependencies = Vec::new();
    let mut in_dependencies = false;
    let mut pending_name: Option<String> = None;
    let mut pending_spec = String::new();
    let mut brace_balance = 0_i32;

    for (index, line) in source.lines().enumerate() {
        let trimmed = line.trim();

        if let Some(name) = &pending_name {
            if !trimmed.is_empty() {
                if !pending_spec.is_empty() {
                    pending_spec.push(' ');
                }
                pending_spec.push_str(trimmed);
                brace_balance += brace_delta(trimmed);
            }

            if brace_balance <= 0 {
                let dependency_name = name.clone();
                let dependency_path = extract_inline_path(&dependency_name, &pending_spec)?;
                dependencies.push(ManifestDependency {
                    name: dependency_name,
                    path: dependency_path,
                });
                pending_name = None;
                pending_spec.clear();
            }

            continue;
        }

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_dependencies = trimmed == "[dependencies]";
            continue;
        }

        if !in_dependencies {
            continue;
        }

        let Some((raw_name, raw_value)) = trimmed.split_once('=') else {
            continue;
        };
        let dependency_name = raw_name.trim();
        if !dependency_name.starts_with("block-") {
            continue;
        }

        let value = raw_value.trim();
        if !value.starts_with('{') {
            return Err(CodegenError::InvalidManifest(format!(
                "catalog dependency {dependency_name} on line {} must use an inline table",
                index + 1
            )));
        }

        brace_balance = brace_delta(value);
        if brace_balance > 0 {
            pending_name = Some(dependency_name.to_string());
            pending_spec.push_str(value);
            continue;
        }

        let dependency_path = extract_inline_path(dependency_name, value)?;
        dependencies.push(ManifestDependency {
            name: dependency_name.to_string(),
            path: dependency_path,
        });
    }

    if let Some(dependency_name) = pending_name {
        return Err(CodegenError::InvalidManifest(format!(
            "catalog dependency {dependency_name} has an unterminated inline table"
        )));
    }

    Ok(dependencies)
}

fn extract_inline_path(dependency_name: &str, value: &str) -> Result<PathBuf, CodegenError> {
    let Some(path_index) = value.find("path") else {
        return Err(CodegenError::MissingPathDependency {
            dependency_name: dependency_name.to_string(),
        });
    };
    let path_value = &value[path_index + "path".len()..];
    let Some((_, after_equals)) = path_value.split_once('=') else {
        return Err(CodegenError::MissingPathDependency {
            dependency_name: dependency_name.to_string(),
        });
    };
    let after_equals = after_equals.trim();
    let Some(stripped) = after_equals.strip_prefix('"') else {
        return Err(CodegenError::MissingPathDependency {
            dependency_name: dependency_name.to_string(),
        });
    };
    let Some(end_quote) = stripped.find('"') else {
        return Err(CodegenError::MissingPathDependency {
            dependency_name: dependency_name.to_string(),
        });
    };

    Ok(PathBuf::from(&stripped[..end_quote]))
}

fn resolve_dependency_dir(manifest_dir: &Path, dependency_path: &Path) -> PathBuf {
    if dependency_path.is_absolute() {
        dependency_path.to_path_buf()
    } else {
        manifest_dir.join(dependency_path)
    }
}

fn dependency_module_name(dependency_name: &str) -> String {
    dependency_name.replace('-', "_")
}

fn brace_delta(value: &str) -> i32 {
    value.chars().fold(0_i32, |balance, current| match current {
        '{' => balance + 1,
        '}' => balance - 1,
        _ => balance,
    })
}
