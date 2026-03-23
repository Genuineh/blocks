use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageKind {
    Block,
    Moc,
    Bcl,
}

impl PackageKind {
    pub fn descriptor_filename(self) -> &'static str {
        match self {
            Self::Block => "block.yaml",
            Self::Moc => "moc.yaml",
            Self::Bcl => "moc.bcl",
        }
    }
}

impl std::fmt::Display for PackageKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Block => write!(f, "block"),
            Self::Moc => write!(f, "moc"),
            Self::Bcl => write!(f, "bcl"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PackageDescriptor {
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PackageDependency {
    pub id: String,
    pub kind: PackageKind,
    pub req: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_hint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PackageSource {
    #[serde(rename = "type")]
    pub source_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageManifest {
    pub api_version: String,
    pub kind: PackageKind,
    pub id: String,
    pub version: String,
    pub descriptor: PackageDescriptor,
    #[serde(default)]
    pub dependencies: Vec<PackageDependency>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<PackageSource>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<BTreeMap<String, serde_yaml::Value>>,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra_top_level: BTreeMap<String, serde_yaml::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ResolvedDependency {
    pub id: String,
    pub kind: PackageKind,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LockedSource {
    #[serde(rename = "type")]
    pub source_type: String,
    pub location: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LockedPackage {
    pub id: String,
    pub kind: PackageKind,
    pub version: String,
    pub source: LockedSource,
    pub descriptor_path: String,
    #[serde(default)]
    pub dependencies: Vec<ResolvedDependency>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageLock {
    pub version: u32,
    pub root: ResolvedDependency,
    #[serde(default)]
    pub providers: Vec<String>,
    #[serde(default)]
    pub resolved: Vec<LockedPackage>,
}

impl PackageLock {
    pub fn canonicalize(&mut self) {
        self.providers.sort();
        self.providers.dedup();
        for package in &mut self.resolved {
            package.dependencies.sort();
        }
        self.resolved.sort();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedPackage {
    pub manifest: PackageManifest,
    pub manifest_path: PathBuf,
    pub package_root: PathBuf,
    pub bridge_mode: bool,
}

#[derive(Debug, Error)]
pub enum PackageLoadError {
    #[error("package root does not exist: {0}")]
    MissingRoot(PathBuf),
    #[error("package manifest is invalid: {0}")]
    InvalidManifest(String),
    #[error("failed to read package manifest {path}: {source}")]
    ReadManifest {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse package manifest {path}: {source}")]
    ParseManifest {
        path: PathBuf,
        #[source]
        source: serde_yaml::Error,
    },
    #[error("failed to read legacy descriptor {path}: {source}")]
    ReadLegacyDescriptor {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse legacy descriptor {path}: {source}")]
    ParseLegacyDescriptor {
        path: PathBuf,
        #[source]
        source: serde_yaml::Error,
    },
}

pub fn load_package(root_or_manifest: impl AsRef<Path>) -> Result<LoadedPackage, PackageLoadError> {
    let path = root_or_manifest.as_ref();
    let (package_root, manifest_path) = if path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name == "package.yaml")
    {
        (
            path.parent()
                .ok_or_else(|| PackageLoadError::MissingRoot(path.to_path_buf()))?
                .to_path_buf(),
            path.to_path_buf(),
        )
    } else {
        (path.to_path_buf(), path.join("package.yaml"))
    };

    if !package_root.exists() {
        return Err(PackageLoadError::MissingRoot(package_root));
    }

    if manifest_path.is_file() {
        let manifest = read_package_manifest(&manifest_path)?;
        validate_manifest(&manifest, &package_root)?;
        return Ok(LoadedPackage {
            manifest,
            manifest_path,
            package_root,
            bridge_mode: false,
        });
    }

    load_legacy_bridge(&package_root)
}

pub fn read_package_manifest(path: &Path) -> Result<PackageManifest, PackageLoadError> {
    let source = fs::read_to_string(path).map_err(|source| PackageLoadError::ReadManifest {
        path: path.to_path_buf(),
        source,
    })?;
    serde_yaml::from_str(&source).map_err(|source| PackageLoadError::ParseManifest {
        path: path.to_path_buf(),
        source,
    })
}

pub fn validate_manifest(manifest: &PackageManifest, root: &Path) -> Result<(), PackageLoadError> {
    validate_manifest_mode(manifest, root, true).map(|_| ())
}

pub fn validate_manifest_compat(
    manifest: &PackageManifest,
    root: &Path,
) -> Result<Vec<String>, PackageLoadError> {
    validate_manifest_mode(manifest, root, false)
}

fn validate_manifest_mode(
    manifest: &PackageManifest,
    root: &Path,
    strict: bool,
) -> Result<Vec<String>, PackageLoadError> {
    if manifest.api_version != "blocks.pkg/v1" {
        return Err(PackageLoadError::InvalidManifest(
            "api_version must be blocks.pkg/v1".to_string(),
        ));
    }
    if !is_valid_identifier(&manifest.id) {
        return Err(PackageLoadError::InvalidManifest(
            "id must use lowercase dot-separated segments".to_string(),
        ));
    }
    if !is_valid_version(&manifest.version) {
        return Err(PackageLoadError::InvalidManifest(
            "version must use semver-like major.minor.patch".to_string(),
        ));
    }
    if manifest.descriptor.path != manifest.kind.descriptor_filename() {
        return Err(PackageLoadError::InvalidManifest(format!(
            "descriptor.path must be {} for kind {}",
            manifest.kind.descriptor_filename(),
            manifest.kind
        )));
    }
    let descriptor_path = root.join(&manifest.descriptor.path);
    if !descriptor_path.is_file() {
        return Err(PackageLoadError::InvalidManifest(format!(
            "descriptor path does not exist: {}",
            descriptor_path.display()
        )));
    }
    let mut seen = BTreeMap::new();
    for dependency in &manifest.dependencies {
        if dependency.id == manifest.id {
            return Err(PackageLoadError::InvalidManifest(
                "package may not depend on itself".to_string(),
            ));
        }
        if !is_valid_identifier(&dependency.id) {
            return Err(PackageLoadError::InvalidManifest(format!(
                "dependency id is invalid: {}",
                dependency.id
            )));
        }
        let key = (dependency.id.clone(), dependency.kind);
        if seen.insert(key, ()).is_some() {
            return Err(PackageLoadError::InvalidManifest(
                "duplicate dependency with same id and kind".to_string(),
            ));
        }
    }
    if manifest.extra_top_level.is_empty() {
        return Ok(Vec::new());
    }

    let unknown_keys = manifest
        .extra_top_level
        .keys()
        .cloned()
        .collect::<Vec<_>>()
        .join(", ");
    if strict {
        return Err(PackageLoadError::InvalidManifest(format!(
            "unknown top-level key(s): {unknown_keys}"
        )));
    }
    Ok(vec![format!(
        "unknown key(s) preserved in compatibility mode: {unknown_keys}"
    )])
}

pub fn render_manifest_yaml(manifest: &PackageManifest) -> Result<String, PackageLoadError> {
    serde_yaml::to_string(manifest).map_err(|error| {
        PackageLoadError::InvalidManifest(format!("failed to render package manifest: {error}"))
    })
}

pub fn render_lock_yaml(lock: &PackageLock) -> Result<String, PackageLoadError> {
    let mut lock = lock.clone();
    lock.canonicalize();
    serde_yaml::to_string(&lock).map_err(|error| {
        PackageLoadError::InvalidManifest(format!("failed to render lockfile: {error}"))
    })
}

fn load_legacy_bridge(package_root: &Path) -> Result<LoadedPackage, PackageLoadError> {
    if let Some(manifest) = load_legacy_moc_bridge(package_root)? {
        let manifest_path = package_root.join("package.yaml");
        return Ok(LoadedPackage {
            manifest,
            manifest_path,
            package_root: package_root.to_path_buf(),
            bridge_mode: true,
        });
    }
    Err(PackageLoadError::InvalidManifest(
        "package.yaml is missing and no supported legacy descriptor bridge was found".to_string(),
    ))
}

fn load_legacy_moc_bridge(
    package_root: &Path,
) -> Result<Option<PackageManifest>, PackageLoadError> {
    let descriptor_path = package_root.join("moc.yaml");
    if !descriptor_path.is_file() {
        return Ok(None);
    }
    let source = fs::read_to_string(&descriptor_path).map_err(|source| {
        PackageLoadError::ReadLegacyDescriptor {
            path: descriptor_path.clone(),
            source,
        }
    })?;
    let value: serde_yaml::Value = serde_yaml::from_str(&source).map_err(|source| {
        PackageLoadError::ParseLegacyDescriptor {
            path: descriptor_path.clone(),
            source,
        }
    })?;
    let id = value
        .get("id")
        .and_then(serde_yaml::Value::as_str)
        .unwrap_or("legacy.bridge")
        .to_string();
    let manifest = PackageManifest {
        api_version: "blocks.pkg/v1".to_string(),
        kind: PackageKind::Moc,
        id,
        version: "0.1.0".to_string(),
        descriptor: PackageDescriptor {
            path: "moc.yaml".to_string(),
        },
        dependencies: Vec::new(),
        source: Some(PackageSource {
            source_type: "workspace".to_string(),
            r#ref: None,
        }),
        metadata: None,
        extra_top_level: BTreeMap::new(),
    };
    Ok(Some(manifest))
}

fn is_valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.split('.').all(|segment| {
            !segment.is_empty()
                && segment
                    .chars()
                    .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
        })
}

fn is_valid_version(value: &str) -> bool {
    let mut parts = value.split('.');
    let parsed = [
        parts.next().and_then(|part| part.parse::<u64>().ok()),
        parts.next().and_then(|part| part.parse::<u64>().ok()),
        parts.next().and_then(|part| part.parse::<u64>().ok()),
    ];
    parsed.iter().all(Option::is_some) && parts.next().is_none()
}
