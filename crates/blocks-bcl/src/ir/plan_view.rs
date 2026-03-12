use serde::Serialize;

use blocks_moc::{ExecutionPlan, MocManifest};

#[derive(Debug, Clone, Serialize)]
pub struct PlanReport {
    pub status: String,
    pub source: String,
    pub moc_id: String,
    pub moc_type: String,
    pub backend_mode: Option<String>,
    pub language: String,
    pub entry: String,
    pub descriptor_only: bool,
    pub uses: PlanUses,
    pub dependencies: Vec<PlanDependency>,
    pub protocols: Vec<PlanProtocol>,
    pub verification: PlanVerification,
    pub acceptance_criteria: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlanUses {
    pub blocks: Vec<String>,
    pub internal_blocks: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlanDependency {
    pub moc: String,
    pub protocol: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlanProtocol {
    pub name: String,
    pub channel: String,
    pub input_fields: Vec<String>,
    pub output_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlanVerification {
    pub commands: Vec<String>,
    pub entry_flow: Option<String>,
    pub plan: Option<ExecutionPlanView>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExecutionPlanView {
    pub flow_id: String,
    pub last_step_id: String,
    pub steps: Vec<PlanStepView>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlanStepView {
    pub id: String,
    pub block: String,
    pub input_bindings: Vec<PlanBindingView>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlanBindingView {
    pub from: String,
    pub to: String,
    pub to_field: String,
}

pub fn build_plan_report(
    source: &str,
    manifest: &MocManifest,
    execution_plan: Option<&ExecutionPlan>,
) -> PlanReport {
    PlanReport {
        status: "ok".to_string(),
        source: source.to_string(),
        moc_id: manifest.id.clone(),
        moc_type: manifest.moc_type.to_string(),
        backend_mode: manifest.backend_mode.map(|mode| mode.to_string()),
        language: manifest.language.clone(),
        entry: manifest.entry.clone(),
        descriptor_only: !manifest.has_validation_flow(),
        uses: PlanUses {
            blocks: manifest.uses.blocks.clone(),
            internal_blocks: manifest.uses.internal_blocks.clone(),
        },
        dependencies: manifest
            .depends_on_mocs
            .iter()
            .map(|dependency| PlanDependency {
                moc: dependency.moc.clone(),
                protocol: dependency.protocol.clone(),
            })
            .collect(),
        protocols: manifest
            .protocols
            .iter()
            .map(|protocol| PlanProtocol {
                name: protocol.name.clone(),
                channel: protocol.channel.clone(),
                input_fields: protocol.input_schema.keys().cloned().collect(),
                output_fields: protocol.output_schema.keys().cloned().collect(),
            })
            .collect(),
        verification: PlanVerification {
            commands: manifest.verification.commands.clone(),
            entry_flow: manifest.verification.entry_flow.clone(),
            plan: execution_plan.map(|plan| ExecutionPlanView {
                flow_id: plan.flow_id.clone(),
                last_step_id: plan.last_step_id.clone(),
                steps: plan
                    .steps
                    .iter()
                    .map(|step| PlanStepView {
                        id: step.id.clone(),
                        block: step.block.clone(),
                        input_bindings: step
                            .input_bindings
                            .iter()
                            .map(|binding| PlanBindingView {
                                from: binding.from.clone(),
                                to: binding.to.clone(),
                                to_field: binding.to_field.clone(),
                            })
                            .collect(),
                    })
                    .collect(),
            }),
        },
        acceptance_criteria: manifest.acceptance_criteria.clone(),
    }
}
