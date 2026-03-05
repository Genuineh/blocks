use std::fs;
use std::path::Path;

pub fn write_block(root: &Path, dir_name: &str, body: &str) {
    fn has_key(source: &str, key: &str) -> bool {
        source
            .lines()
            .any(|line| line.trim_start().starts_with(&format!("{key}:")))
    }

    fn ensure_key(source: &mut String, key: &str, snippet: &str) {
        if !has_key(source, key) {
            source.push_str(snippet);
        }
    }

    let mut content = body.to_string();
    ensure_key(&mut content, "version", "version: 0.1.0\n");
    ensure_key(&mut content, "status", "status: candidate\n");
    ensure_key(&mut content, "owner", "owner: blocks-core-team\n");
    ensure_key(&mut content, "purpose", "purpose: test block\n");
    ensure_key(&mut content, "scope", "scope:\n  - test scope\n");
    ensure_key(&mut content, "non_goals", "non_goals:\n  - test non-goal\n");
    ensure_key(
        &mut content,
        "inputs",
        "inputs:\n  - name: text\n    description: input\n",
    );
    ensure_key(
        &mut content,
        "input_schema",
        "input_schema:\n  text:\n    type: string\n    required: true\n",
    );
    ensure_key(
        &mut content,
        "preconditions",
        "preconditions:\n  - input exists\n",
    );
    ensure_key(
        &mut content,
        "outputs",
        "outputs:\n  - name: text\n    description: output\n",
    );
    ensure_key(
        &mut content,
        "output_schema",
        "output_schema:\n  text:\n    type: string\n    required: true\n",
    );
    ensure_key(
        &mut content,
        "postconditions",
        "postconditions:\n  - output exists\n",
    );
    ensure_key(
        &mut content,
        "dependencies",
        "dependencies:\n  runtime:\n    - std\n",
    );
    ensure_key(&mut content, "side_effects", "side_effects:\n  - none\n");
    ensure_key(&mut content, "timeouts", "timeouts:\n  default_ms: 100\n");
    ensure_key(
        &mut content,
        "resource_limits",
        "resource_limits:\n  memory_mb: 16\n",
    );
    ensure_key(
        &mut content,
        "failure_modes",
        "failure_modes:\n  - id: invalid_input\n    when: invalid input\n",
    );
    ensure_key(
        &mut content,
        "error_codes",
        "error_codes:\n  - invalid_input\n",
    );
    ensure_key(
        &mut content,
        "recovery_strategy",
        "recovery_strategy:\n  - retry\n",
    );
    ensure_key(
        &mut content,
        "verification",
        "verification:\n  automated:\n    - cargo test\n",
    );
    ensure_key(
        &mut content,
        "evaluation",
        "evaluation:\n  quality_gates:\n    - stable\n",
    );
    ensure_key(
        &mut content,
        "acceptance_criteria",
        "acceptance_criteria:\n  - works\n",
    );
    ensure_key(
        &mut content,
        "debug",
        "debug:\n  enabled_in_dev: true\n  emits_structured_logs: true\n  log_fields:\n    - execution_id\n",
    );
    ensure_key(
        &mut content,
        "observe",
        "observe:\n  metrics:\n    - execution_total\n  emits_failure_artifact: true\n  artifact_policy:\n    mode: on_failure\n",
    );
    ensure_key(
        &mut content,
        "errors",
        "errors:\n  taxonomy:\n    - id: invalid_input\n    - id: internal_error\n",
    );

    let block_dir = root.join(dir_name);
    let rust_dir = block_dir.join("rust");
    fs::create_dir_all(&rust_dir).expect("block dir should be created");
    fs::write(block_dir.join("block.yaml"), content).expect("contract should be written");
    fs::write(rust_dir.join("lib.rs"), "// fixture").expect("implementation should be written");
}
