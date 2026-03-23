use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use blocks_contract::{
    ArtifactMode, BlockContract, ImplementationKind, ImplementationTarget, ValidationIssue,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionResult {
    pub output: Value,
    pub record: ExecutionRecord,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionRecord {
    pub block_id: String,
    pub execution_id: String,
    pub trace_id: Option<String>,
    pub success: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExecutionContext {
    pub trace_id: Option<String>,
    pub moc_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HostProfile {
    SyncCli,
    TokioService,
}

impl HostProfile {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SyncCli => "sync-cli",
            Self::TokioService => "tokio-service",
        }
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "sync-cli" => Ok(Self::SyncCli),
            "tokio-service" => Ok(Self::TokioService),
            other => Err(format!("unsupported runtime host profile: {other}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostCapabilities {
    pub host_profile: String,
    pub runtime_model: String,
    pub in_process: bool,
    pub supports_contract_validation: bool,
    pub supports_diagnostics_artifacts: bool,
    pub supports_trace_context: bool,
    pub supports_moc_context: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostCompatibilityReport {
    pub host_profile: String,
    pub status: String,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub capabilities: HostCapabilities,
}

pub struct ExecutionEnvelope<'a> {
    pub contract: &'a BlockContract,
    pub input: &'a Value,
    pub context: &'a ExecutionContext,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticEvent {
    pub timestamp_ms: u128,
    pub level: String,
    pub event: String,
    pub block_id: String,
    pub block_version: String,
    pub execution_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moc_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u128>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiagnosticArtifact {
    pub execution_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moc_id: Option<String>,
    pub block_id: String,
    pub input_snapshot: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_snapshot: Option<Value>,
    pub error: DiagnosticError,
    pub environment: DiagnosticEnvironment,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiagnosticError {
    pub error_id: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticEnvironment {
    pub runtime_mode: String,
    pub implementation_kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{message}")]
pub struct BlockExecutionError {
    message: String,
}

impl BlockExecutionError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

pub trait BlockRunner {
    fn run(&self, block_id: &str, input: &Value) -> Result<Value, BlockExecutionError>;
}

#[derive(Debug, Error)]
pub enum RuntimeHostError {
    #[error("runtime host `{host_profile}` is unavailable: {message}")]
    HostUnavailable {
        host_profile: String,
        message: String,
    },
    #[error("runtime host `{host_profile}` failed: {source}")]
    Execution {
        host_profile: String,
        #[source]
        source: RuntimeError,
    },
}

pub trait RuntimeHost {
    fn profile(&self) -> HostProfile;
    fn capabilities(&self) -> HostCapabilities;
    fn check_contract(&self, contract: &BlockContract) -> HostCompatibilityReport;
    fn execute_envelope(
        &self,
        envelope: &ExecutionEnvelope<'_>,
        runner: &dyn BlockRunner,
    ) -> Result<ExecutionResult, RuntimeHostError>;
}

#[derive(Debug, Clone)]
pub struct SyncCliRuntimeHost {
    runtime: Runtime,
}

#[derive(Debug, Clone)]
pub struct TokioServiceRuntimeHost {
    runtime: Runtime,
}

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error(
        "input validation failed (execution_id: {execution_id}, diagnostics: {diagnostic_path})"
    )]
    InputValidationFailed {
        execution_id: String,
        diagnostic_path: String,
        issues: Vec<ValidationIssue>,
    },
    #[error(
        "block execution failed (execution_id: {execution_id}, diagnostics: {diagnostic_path}). details: [REDACTED]"
    )]
    ExecutionFailed {
        execution_id: String,
        diagnostic_path: String,
        #[source]
        source: BlockExecutionError,
    },
    #[error(
        "output validation failed (execution_id: {execution_id}, diagnostics: {diagnostic_path})"
    )]
    OutputValidationFailed {
        execution_id: String,
        diagnostic_path: String,
        issues: Vec<ValidationIssue>,
    },
}

#[derive(Debug, Clone)]
pub struct Runtime {
    diagnostics_root: PathBuf,
}

#[derive(Debug, Clone, Copy)]
struct FailureArtifactPolicy {
    write_artifact: bool,
    include_input_snapshot: bool,
    include_output_snapshot: bool,
    include_error_report: bool,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            diagnostics_root: default_diagnostics_root(),
        }
    }

    pub fn with_diagnostics_root(path: impl Into<PathBuf>) -> Self {
        Self {
            diagnostics_root: path.into(),
        }
    }

    pub fn diagnostics_root(&self) -> &Path {
        &self.diagnostics_root
    }

    pub fn execute_with_context(
        &self,
        contract: &BlockContract,
        input: &Value,
        runner: &dyn BlockRunner,
        context: &ExecutionContext,
    ) -> Result<ExecutionResult, RuntimeError> {
        let execution_id = generate_execution_id();
        let start = SystemTime::now();
        let trace_id = context.trace_id.clone();
        let moc_id = context.moc_id.clone();
        let block_version = contract
            .version
            .clone()
            .unwrap_or_else(|| "unknown".to_string());
        self.append_event(DiagnosticEvent {
            timestamp_ms: now_ms(),
            level: "INFO".to_string(),
            event: "block.execution.start".to_string(),
            block_id: contract.id.clone(),
            block_version: block_version.clone(),
            execution_id: execution_id.clone(),
            trace_id: trace_id.clone(),
            moc_id: moc_id.clone(),
            duration_ms: None,
            error_id: None,
            message: None,
        });

        if let Err(issues) = contract.validate_input(input) {
            let artifact_policy = failure_artifact_policy(contract);
            let error = RuntimeError::InputValidationFailed {
                execution_id: execution_id.clone(),
                diagnostic_path: failure_diagnostic_path(
                    &self.diagnostics_root,
                    &execution_id,
                    artifact_policy,
                ),
                issues,
            };
            self.handle_failure(
                contract,
                artifact_policy,
                &execution_id,
                trace_id,
                moc_id,
                &contract.id,
                &block_version,
                resolve_error_id(contract, "invalid_input"),
                &error.to_string(),
                input,
                None,
                start.elapsed().ok(),
            );
            return Err(error);
        }

        let output = match runner.run(&contract.id, input) {
            Ok(output) => output,
            Err(source) => {
                let artifact_policy = failure_artifact_policy(contract);
                let error = RuntimeError::ExecutionFailed {
                    execution_id: execution_id.clone(),
                    diagnostic_path: failure_diagnostic_path(
                        &self.diagnostics_root,
                        &execution_id,
                        artifact_policy,
                    ),
                    source,
                };
                self.handle_failure(
                    contract,
                    artifact_policy,
                    &execution_id,
                    trace_id,
                    moc_id,
                    &contract.id,
                    &block_version,
                    resolve_error_id(contract, "internal_error"),
                    &error.to_string(),
                    input,
                    None,
                    start.elapsed().ok(),
                );
                return Err(error);
            }
        };

        if let Err(issues) = contract.validate_output(&output) {
            let artifact_policy = failure_artifact_policy(contract);
            let error = RuntimeError::OutputValidationFailed {
                execution_id: execution_id.clone(),
                diagnostic_path: failure_diagnostic_path(
                    &self.diagnostics_root,
                    &execution_id,
                    artifact_policy,
                ),
                issues,
            };
            self.handle_failure(
                contract,
                artifact_policy,
                &execution_id,
                trace_id.clone(),
                moc_id.clone(),
                &contract.id,
                &block_version,
                resolve_error_id(contract, "invalid_output"),
                &error.to_string(),
                input,
                Some(&output),
                start.elapsed().ok(),
            );
            return Err(error);
        }

        self.append_event(DiagnosticEvent {
            timestamp_ms: now_ms(),
            level: "INFO".to_string(),
            event: "block.execution.success".to_string(),
            block_id: contract.id.clone(),
            block_version: block_version.clone(),
            execution_id: execution_id.clone(),
            trace_id: trace_id.clone(),
            moc_id,
            duration_ms: start.elapsed().ok().map(|duration| duration.as_millis()),
            error_id: None,
            message: None,
        });

        Ok(ExecutionResult {
            output,
            record: ExecutionRecord {
                block_id: contract.id.clone(),
                execution_id,
                trace_id,
                success: true,
            },
        })
    }

    pub fn execute(
        &self,
        contract: &BlockContract,
        input: &Value,
        runner: &dyn BlockRunner,
    ) -> Result<ExecutionResult, RuntimeError> {
        self.execute_with_context(contract, input, runner, &ExecutionContext::default())
    }

    fn handle_failure(
        &self,
        contract: &BlockContract,
        artifact_policy: FailureArtifactPolicy,
        execution_id: &str,
        trace_id: Option<String>,
        moc_id: Option<String>,
        block_id: &str,
        block_version: &str,
        error_id: String,
        message: &str,
        input: &Value,
        output: Option<&Value>,
        duration: Option<Duration>,
    ) {
        self.append_event(DiagnosticEvent {
            timestamp_ms: now_ms(),
            level: "ERROR".to_string(),
            event: "block.execution.failure".to_string(),
            block_id: block_id.to_string(),
            block_version: block_version.to_string(),
            execution_id: execution_id.to_string(),
            trace_id: trace_id.clone(),
            moc_id: moc_id.clone(),
            duration_ms: duration.map(|d| d.as_millis()),
            error_id: Some(error_id.clone()),
            message: Some(message.to_string()),
        });

        if !artifact_policy.write_artifact {
            return;
        }

        let artifact = DiagnosticArtifact {
            execution_id: execution_id.to_string(),
            trace_id,
            moc_id,
            block_id: block_id.to_string(),
            input_snapshot: snapshot_value(input, artifact_policy.include_input_snapshot),
            output_snapshot: output.and_then(|value| {
                artifact_policy
                    .include_output_snapshot
                    .then(|| redact_value(value))
            }),
            error: DiagnosticError {
                error_id,
                message: diagnostic_error_message(message, artifact_policy.include_error_report),
            },
            environment: DiagnosticEnvironment {
                runtime_mode: "dev".to_string(),
                implementation_kind: diagnostic_implementation_kind(contract).to_string(),
            },
        };
        self.write_artifact(&artifact);
    }

    fn append_event(&self, event: DiagnosticEvent) {
        if fs::create_dir_all(&self.diagnostics_root).is_err() {
            return;
        }

        let events_file = self.diagnostics_root.join("events.jsonl");
        let mut file = match OpenOptions::new()
            .create(true)
            .append(true)
            .open(events_file)
        {
            Ok(file) => file,
            Err(_) => return,
        };
        if let Ok(serialized) = serde_json::to_string(&event) {
            let _ = writeln!(file, "{serialized}");
        }
    }

    fn write_artifact(&self, artifact: &DiagnosticArtifact) {
        let artifact_dir = self.diagnostics_root.join("artifacts");
        if fs::create_dir_all(&artifact_dir).is_err() {
            return;
        }

        let path = artifact_dir.join(format!("{}.json", artifact.execution_id));
        if let Ok(serialized) = serde_json::to_string_pretty(artifact) {
            let _ = fs::write(path, serialized);
        }
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncCliRuntimeHost {
    pub fn new() -> Self {
        Self {
            runtime: Runtime::new(),
        }
    }

    pub fn with_runtime(runtime: Runtime) -> Self {
        Self { runtime }
    }
}

impl Default for SyncCliRuntimeHost {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeHost for SyncCliRuntimeHost {
    fn profile(&self) -> HostProfile {
        HostProfile::SyncCli
    }

    fn capabilities(&self) -> HostCapabilities {
        host_capabilities(self.profile(), "in_process_sync")
    }

    fn check_contract(&self, contract: &BlockContract) -> HostCompatibilityReport {
        host_compatibility_report(self.profile(), contract, self.capabilities())
    }

    fn execute_envelope(
        &self,
        envelope: &ExecutionEnvelope<'_>,
        runner: &dyn BlockRunner,
    ) -> Result<ExecutionResult, RuntimeHostError> {
        let report = self.check_contract(envelope.contract);
        if report.status == "error" {
            return Err(RuntimeHostError::HostUnavailable {
                host_profile: report.host_profile,
                message: report.errors.join("; "),
            });
        }
        self.runtime
            .execute_with_context(envelope.contract, envelope.input, runner, envelope.context)
            .map_err(|source| RuntimeHostError::Execution {
                host_profile: self.profile().as_str().to_string(),
                source,
            })
    }
}

impl TokioServiceRuntimeHost {
    pub fn new() -> Self {
        Self {
            runtime: Runtime::new(),
        }
    }

    pub fn with_runtime(runtime: Runtime) -> Self {
        Self { runtime }
    }
}

impl Default for TokioServiceRuntimeHost {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeHost for TokioServiceRuntimeHost {
    fn profile(&self) -> HostProfile {
        HostProfile::TokioService
    }

    fn capabilities(&self) -> HostCapabilities {
        host_capabilities(self.profile(), "tokio_current_thread")
    }

    fn check_contract(&self, contract: &BlockContract) -> HostCompatibilityReport {
        host_compatibility_report(self.profile(), contract, self.capabilities())
    }

    fn execute_envelope(
        &self,
        envelope: &ExecutionEnvelope<'_>,
        runner: &dyn BlockRunner,
    ) -> Result<ExecutionResult, RuntimeHostError> {
        let report = self.check_contract(envelope.contract);
        if report.status == "error" {
            return Err(RuntimeHostError::HostUnavailable {
                host_profile: report.host_profile,
                message: report.errors.join("; "),
            });
        }
        let tokio_runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .map_err(|error| RuntimeHostError::HostUnavailable {
                host_profile: self.profile().as_str().to_string(),
                message: error.to_string(),
            })?;
        tokio_runtime
            .block_on(async {
                self.runtime.execute_with_context(
                    envelope.contract,
                    envelope.input,
                    runner,
                    envelope.context,
                )
            })
            .map_err(|source| RuntimeHostError::Execution {
                host_profile: self.profile().as_str().to_string(),
                source,
            })
    }
}

fn host_capabilities(profile: HostProfile, runtime_model: &str) -> HostCapabilities {
    HostCapabilities {
        host_profile: profile.as_str().to_string(),
        runtime_model: runtime_model.to_string(),
        in_process: true,
        supports_contract_validation: true,
        supports_diagnostics_artifacts: true,
        supports_trace_context: true,
        supports_moc_context: true,
    }
}

fn host_compatibility_report(
    profile: HostProfile,
    contract: &BlockContract,
    capabilities: HostCapabilities,
) -> HostCompatibilityReport {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    match &contract.implementation {
        Some(implementation) => {
            if implementation.kind != ImplementationKind::Rust {
                errors.push(format!(
                    "runtime host `{}` only supports rust implementations in Phase 3",
                    profile.as_str()
                ));
            }
            if implementation.target == ImplementationTarget::Frontend {
                errors.push(format!(
                    "runtime host `{}` does not support frontend targets",
                    profile.as_str()
                ));
            }
            if profile == HostProfile::TokioService
                && implementation.target == ImplementationTarget::Shared
            {
                warnings.push(
                    "shared rust target is running through the tokio service compatibility profile"
                        .to_string(),
                );
            }
        }
        None => errors.push("block contract is missing implementation metadata".to_string()),
    }

    HostCompatibilityReport {
        host_profile: profile.as_str().to_string(),
        status: if errors.is_empty() {
            if warnings.is_empty() { "ok" } else { "warn" }
        } else {
            "error"
        }
        .to_string(),
        warnings,
        errors,
        capabilities,
    }
}

pub fn default_diagnostics_root() -> PathBuf {
    PathBuf::from(".blocks").join("diagnostics")
}

pub fn supported_host_profiles() -> &'static [HostProfile] {
    &[HostProfile::SyncCli, HostProfile::TokioService]
}

pub fn generate_trace_id() -> String {
    generate_id("trace")
}

pub fn read_diagnostic_events(diagnostics_root: &Path) -> Result<Vec<DiagnosticEvent>, String> {
    let events_path = diagnostics_root.join("events.jsonl");
    let source = fs::read_to_string(&events_path).map_err(|error| {
        format!(
            "failed to read diagnostics events {}: {error}",
            events_path.display()
        )
    })?;

    source
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            serde_json::from_str::<DiagnosticEvent>(line)
                .map_err(|error| format!("failed to parse diagnostic event: {error}"))
        })
        .collect()
}

pub fn read_diagnostic_artifact(
    diagnostics_root: &Path,
    execution_id: &str,
) -> Result<Option<DiagnosticArtifact>, String> {
    let artifact_path = diagnostics_root
        .join("artifacts")
        .join(format!("{execution_id}.json"));
    if !artifact_path.is_file() {
        return Ok(None);
    }

    let source = fs::read_to_string(&artifact_path).map_err(|error| {
        format!(
            "failed to read diagnostic artifact {}: {error}",
            artifact_path.display()
        )
    })?;
    let artifact = serde_json::from_str::<DiagnosticArtifact>(&source).map_err(|error| {
        format!(
            "failed to parse diagnostic artifact {}: {error}",
            artifact_path.display()
        )
    })?;
    Ok(Some(artifact))
}

fn generate_execution_id() -> String {
    generate_id("exec")
}

fn generate_id(prefix: &str) -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    let timestamp = now_ms();
    let sequence = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{prefix}-{timestamp:x}-{sequence:x}")
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis())
}

fn redact_value(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(key, value)| {
                    if should_redact_key(key) {
                        (key.clone(), Value::String("***REDACTED***".to_string()))
                    } else {
                        (key.clone(), redact_value(value))
                    }
                })
                .collect::<Map<String, Value>>(),
        ),
        Value::Array(values) => Value::Array(values.iter().map(redact_value).collect()),
        Value::String(text) if is_bearer_token(text) => Value::String("***REDACTED***".to_string()),
        other => other.clone(),
    }
}

fn should_redact_key(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    lower.contains("password")
        || lower.contains("token")
        || lower.contains("secret")
        || lower.contains("authorization")
        || lower.contains("api_key")
        || lower.contains("api-key")
}

fn is_bearer_token(value: &str) -> bool {
    value
        .get(..7)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("Bearer "))
}

fn diagnostic_artifact_path(diagnostics_root: &Path, execution_id: &str) -> String {
    diagnostics_root
        .join("artifacts")
        .join(format!("{execution_id}.json"))
        .display()
        .to_string()
}

fn diagnostics_events_path(diagnostics_root: &Path) -> String {
    diagnostics_root.join("events.jsonl").display().to_string()
}

fn failure_diagnostic_path(
    diagnostics_root: &Path,
    execution_id: &str,
    artifact_policy: FailureArtifactPolicy,
) -> String {
    if artifact_policy.write_artifact {
        diagnostic_artifact_path(diagnostics_root, execution_id)
    } else {
        diagnostics_events_path(diagnostics_root)
    }
}

fn resolve_error_id(contract: &BlockContract, preferred_id: &str) -> String {
    let taxonomy = contract
        .errors
        .as_ref()
        .map(|errors| errors.taxonomy.as_slice())
        .unwrap_or(&[]);
    if taxonomy.is_empty() {
        return preferred_id.to_string();
    }
    if taxonomy.iter().any(|entry| entry.id == preferred_id) {
        return preferred_id.to_string();
    }
    if taxonomy.iter().any(|entry| entry.id == "internal_error") {
        return "internal_error".to_string();
    }
    format!("runtime_fallback_{preferred_id}")
}

fn failure_artifact_policy(contract: &BlockContract) -> FailureArtifactPolicy {
    let Some(observe) = contract.observe.as_ref() else {
        return FailureArtifactPolicy {
            write_artifact: true,
            include_input_snapshot: true,
            include_output_snapshot: true,
            include_error_report: true,
        };
    };

    if !observe.emits_failure_artifact {
        return FailureArtifactPolicy {
            write_artifact: false,
            include_input_snapshot: false,
            include_output_snapshot: false,
            include_error_report: false,
        };
    }

    let Some(policy) = observe.artifact_policy.as_ref() else {
        return FailureArtifactPolicy {
            write_artifact: true,
            include_input_snapshot: true,
            include_output_snapshot: true,
            include_error_report: true,
        };
    };

    if policy.mode == ArtifactMode::Never {
        return FailureArtifactPolicy {
            write_artifact: false,
            include_input_snapshot: false,
            include_output_snapshot: false,
            include_error_report: false,
        };
    }

    let minimum = policy.on_failure_minimum.as_ref();
    let include_output_snapshot = !matches!(
        minimum.and_then(|minimum| minimum.include_output_snapshot.as_deref()),
        Some("never")
    );

    FailureArtifactPolicy {
        write_artifact: matches!(policy.mode, ArtifactMode::Always | ArtifactMode::OnFailure),
        include_input_snapshot: minimum.is_none_or(|minimum| minimum.include_input_snapshot),
        include_output_snapshot,
        include_error_report: minimum.is_none_or(|minimum| minimum.include_error_report),
    }
}

fn snapshot_value(value: &Value, include_snapshot: bool) -> Value {
    if include_snapshot {
        redact_value(value)
    } else {
        Value::Null
    }
}

fn diagnostic_error_message(message: &str, include_error_report: bool) -> String {
    if include_error_report {
        message.to_string()
    } else {
        "suppressed by artifact policy".to_string()
    }
}

fn diagnostic_implementation_kind(contract: &BlockContract) -> &str {
    match contract.implementation.as_ref().map(|value| value.kind) {
        Some(implementation_kind) => implementation_kind.as_str(),
        None => "runtime_wrapper",
    }
}

trait ImplementationKindLabel {
    fn as_str(self) -> &'static str;
}

impl ImplementationKindLabel for blocks_contract::ImplementationKind {
    fn as_str(self) -> &'static str {
        match self {
            blocks_contract::ImplementationKind::Rust => "rust",
            blocks_contract::ImplementationKind::TauriTs => "tauri_ts",
        }
    }
}
