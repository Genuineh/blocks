use crate::diagnostics::SpanRange;

#[derive(Debug, Clone)]
pub struct BclDocument {
    pub moc_id: String,
    pub moc_id_span: SpanRange,
    pub file_span: SpanRange,
    pub name: Option<SpannedString>,
    pub type_spec: Option<TypeSpec>,
    pub language: Option<SpannedIdent>,
    pub entry: Option<SpannedString>,
    pub input_schema: Vec<SchemaFieldDecl>,
    pub output_schema: Vec<SchemaFieldDecl>,
    pub uses: UsesBlock,
    pub dependencies: Vec<DependencyDecl>,
    pub protocols: Vec<ProtocolDecl>,
    pub verification: VerificationDecl,
    pub acceptance: Vec<SpannedString>,
}

#[derive(Debug, Clone)]
pub struct SpannedString {
    pub value: String,
    pub span: SpanRange,
}

#[derive(Debug, Clone)]
pub struct SpannedIdent {
    pub value: String,
    pub span: SpanRange,
}

#[derive(Debug, Clone)]
pub struct TypeSpec {
    pub moc_type: String,
    pub backend_mode: Option<String>,
    pub span: SpanRange,
}

#[derive(Debug, Clone, Default)]
pub struct UsesBlock {
    pub blocks: Vec<SpannedIdent>,
    pub internal_blocks: Vec<SpannedIdent>,
    pub span: Option<SpanRange>,
}

#[derive(Debug, Clone)]
pub struct DependencyDecl {
    pub moc: String,
    pub protocol: String,
    pub span: SpanRange,
}

#[derive(Debug, Clone)]
pub struct ProtocolDecl {
    pub name: String,
    pub name_span: SpanRange,
    pub channel: SpannedIdent,
    pub input_fields: Vec<SchemaFieldDecl>,
    pub output_fields: Vec<SchemaFieldDecl>,
    pub span: SpanRange,
}

#[derive(Debug, Clone)]
pub struct SchemaFieldDecl {
    pub name: String,
    pub field_type: String,
    pub required: bool,
    pub span: SpanRange,
}

#[derive(Debug, Clone, Default)]
pub struct VerificationDecl {
    pub commands: Vec<SpannedString>,
    pub flows: Vec<FlowDecl>,
    pub span: Option<SpanRange>,
}

#[derive(Debug, Clone)]
pub struct FlowDecl {
    pub id: String,
    pub span: SpanRange,
    pub is_entry: bool,
    pub steps: Vec<StepDecl>,
    pub binds: Vec<BindDecl>,
}

#[derive(Debug, Clone)]
pub struct StepDecl {
    pub id: String,
    pub block: String,
    pub span: SpanRange,
}

#[derive(Debug, Clone)]
pub struct BindDecl {
    pub from: String,
    pub to: String,
    pub span: SpanRange,
}
