use std::iter::Peekable;
use std::str::Chars;

use crate::diagnostics::{RuleResult, SpanRange, ValidateReport};

use super::ast::{
    BclDocument, BindDecl, DependencyDecl, FlowDecl, GuardClause, ProtocolDecl,
    RecoverClause, SchemaFieldDecl, SpannedIdent, SpannedString, StepDecl, TypeSpec,
    UsesBlock, VerificationDecl,
};

#[derive(Debug, Clone)]
struct Token {
    kind: TokenKind,
    span: SpanRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TokenKind {
    Ident(String),
    String(String),
    LBrace,
    RBrace,
    LParen,
    RParen,
    Equals,
    Colon,
    Semicolon,
    Dot,
    Arrow,
}

pub fn parse(source_path: &str, source: &str) -> Result<BclDocument, ValidateReport> {
    let tokens = tokenize(source_path, source)?;
    Parser::new(source_path, tokens).parse_document()
}

struct Parser<'a> {
    source_path: &'a str,
    tokens: Vec<Token>,
    index: usize,
}

impl<'a> Parser<'a> {
    fn new(source_path: &'a str, tokens: Vec<Token>) -> Self {
        Self {
            source_path,
            tokens,
            index: 0,
        }
    }

    fn parse_document(mut self) -> Result<BclDocument, ValidateReport> {
        self.expect_keyword("moc")?;
        let moc_id_token = self.expect_ident_token()?;
        let moc_id = match &moc_id_token.kind {
            TokenKind::Ident(value) => value.clone(),
            _ => unreachable!(),
        };
        let file_start = moc_id_token.span.clone();
        self.expect_simple(TokenKind::LBrace)?;

        let mut document = BclDocument {
            moc_id,
            moc_id_span: moc_id_token.span,
            file_span: file_start,
            name: None,
            type_spec: None,
            language: None,
            entry: None,
            input_schema: Vec::new(),
            output_schema: Vec::new(),
            uses: UsesBlock::default(),
            dependencies: Vec::new(),
            protocols: Vec::new(),
            verification: VerificationDecl::default(),
            acceptance: Vec::new(),
        };

        while !self.check_simple(TokenKind::RBrace) {
            let keyword = self.expect_ident_token()?;
            let name = match &keyword.kind {
                TokenKind::Ident(value) => value.as_str(),
                _ => unreachable!(),
            };

            match name {
                "name" => {
                    document.name = Some(self.parse_string_stmt()?);
                }
                "type" => {
                    document.type_spec = Some(self.parse_type_stmt(keyword.span.clone())?);
                }
                "language" => {
                    document.language = Some(self.parse_ident_stmt()?);
                }
                "entry" => {
                    document.entry = Some(self.parse_string_stmt()?);
                }
                "input" => {
                    document.input_schema = self.parse_schema_block()?;
                }
                "output" => {
                    document.output_schema = self.parse_schema_block()?;
                }
                "uses" => {
                    document.uses = self.parse_uses_block()?;
                }
                "depends_on_mocs" => {
                    document.dependencies = self.parse_dependency_block()?;
                }
                "protocols" => {
                    document.protocols = self.parse_protocols_block()?;
                }
                "verification" => {
                    document.verification = self.parse_verification_block()?;
                }
                "accept" => {
                    document.acceptance.push(self.parse_string_stmt()?);
                }
                "product" => {
                    return Err(self.unsupported_error(
                        keyword.span,
                        format!("unsupported BCL construct in MVP: {name}"),
                    ));
                }
                _ => {
                    return Err(self.syntax_error(
                        keyword.span,
                        format!("unknown statement: {name}"),
                        Some("use one of: name, type, language, entry, input, output, uses, depends_on_mocs, protocols, verification, accept".to_string()),
                    ));
                }
            }
        }

        let file_end = self.expect_simple(TokenKind::RBrace)?;
        document.file_span = SpanRange::new(1, 1, file_end.end_line, file_end.end_column);
        if self.index != self.tokens.len() {
            let span = self.tokens[self.index].span.clone();
            return Err(self.syntax_error(span, "unexpected trailing tokens".to_string(), None));
        }

        Ok(document)
    }

    fn parse_string_stmt(&mut self) -> Result<SpannedString, ValidateReport> {
        let token = self.expect_string_token()?;
        self.expect_simple(TokenKind::Semicolon)?;
        match token.kind {
            TokenKind::String(value) => Ok(SpannedString {
                value,
                span: token.span,
            }),
            _ => unreachable!(),
        }
    }

    fn parse_ident_stmt(&mut self) -> Result<SpannedIdent, ValidateReport> {
        let token = self.expect_ident_token()?;
        self.expect_simple(TokenKind::Semicolon)?;
        match token.kind {
            TokenKind::Ident(value) => Ok(SpannedIdent {
                value,
                span: token.span,
            }),
            _ => unreachable!(),
        }
    }

    fn parse_type_stmt(&mut self, start_span: SpanRange) -> Result<TypeSpec, ValidateReport> {
        let moc_type = self.expect_ident_token()?;
        let moc_type_name = match &moc_type.kind {
            TokenKind::Ident(value) => value.clone(),
            _ => unreachable!(),
        };
        let mut backend_mode = None;
        if self.check_simple(TokenKind::LParen) {
            self.expect_simple(TokenKind::LParen)?;
            let mode_token = self.expect_ident_token()?;
            backend_mode = Some(match mode_token.kind {
                TokenKind::Ident(value) => value,
                _ => unreachable!(),
            });
            self.expect_simple(TokenKind::RParen)?;
        }
        let semicolon = self.expect_simple(TokenKind::Semicolon)?;
        Ok(TypeSpec {
            moc_type: moc_type_name,
            backend_mode,
            span: SpanRange::new(
                start_span.line,
                start_span.column,
                semicolon.end_line,
                semicolon.end_column,
            ),
        })
    }

    fn parse_uses_block(&mut self) -> Result<UsesBlock, ValidateReport> {
        let start = self.expect_simple(TokenKind::LBrace)?;
        let mut uses = UsesBlock {
            blocks: Vec::new(),
            internal_blocks: Vec::new(),
            span: Some(start.clone()),
        };
        while !self.check_simple(TokenKind::RBrace) {
            let item = self.expect_ident_token()?;
            let item_name = match &item.kind {
                TokenKind::Ident(value) => value.as_str(),
                _ => unreachable!(),
            };
            let ident = self.parse_dotted_ident()?;
            self.expect_simple(TokenKind::Semicolon)?;
            match item_name {
                "block" => uses.blocks.push(ident),
                "internal_block" => uses.internal_blocks.push(ident),
                _ => {
                    return Err(self.syntax_error(
                        item.span,
                        format!("unknown uses item: {item_name}"),
                        Some("expected `block` or `internal_block`".to_string()),
                    ));
                }
            }
        }
        let end = self.expect_simple(TokenKind::RBrace)?;
        uses.span = Some(SpanRange::new(
            start.line,
            start.column,
            end.end_line,
            end.end_column,
        ));
        Ok(uses)
    }

    fn parse_dependency_block(&mut self) -> Result<Vec<DependencyDecl>, ValidateReport> {
        self.expect_simple(TokenKind::LBrace)?;
        let mut dependencies = Vec::new();
        while !self.check_simple(TokenKind::RBrace) {
            let start = self.expect_keyword("moc")?;
            let moc = self.expect_string_token()?;
            self.expect_keyword("via")?;
            let protocol = self.parse_dotted_ident()?;
            let end = self.expect_simple(TokenKind::Semicolon)?;
            let moc_value = match moc.kind {
                TokenKind::String(value) => value,
                _ => unreachable!(),
            };
            dependencies.push(DependencyDecl {
                moc: moc_value,
                protocol: protocol.value,
                span: SpanRange::new(start.line, start.column, end.end_line, end.end_column),
            });
        }
        self.expect_simple(TokenKind::RBrace)?;
        Ok(dependencies)
    }

    fn parse_protocols_block(&mut self) -> Result<Vec<ProtocolDecl>, ValidateReport> {
        self.expect_simple(TokenKind::LBrace)?;
        let mut protocols = Vec::new();
        while !self.check_simple(TokenKind::RBrace) {
            let start = self.expect_keyword("protocol")?;
            let name = self.parse_dotted_ident()?;
            self.expect_simple(TokenKind::LBrace)?;
            self.expect_keyword("channel")?;
            let channel = self.expect_ident_stmt_in_block()?;
            self.expect_keyword("input")?;
            let input_fields = self.parse_schema_block()?;
            self.expect_keyword("output")?;
            let output_fields = self.parse_schema_block()?;
            let end = self.expect_simple(TokenKind::RBrace)?;
            protocols.push(ProtocolDecl {
                name: name.value,
                name_span: name.span,
                channel,
                input_fields,
                output_fields,
                span: SpanRange::new(start.line, start.column, end.end_line, end.end_column),
            });
        }
        self.expect_simple(TokenKind::RBrace)?;
        Ok(protocols)
    }

    fn parse_schema_block(&mut self) -> Result<Vec<SchemaFieldDecl>, ValidateReport> {
        self.expect_simple(TokenKind::LBrace)?;
        let mut fields = Vec::new();
        while !self.check_simple(TokenKind::RBrace) {
            let start = self.expect_ident_token()?;
            let field_name = match &start.kind {
                TokenKind::Ident(value) => value.clone(),
                _ => unreachable!(),
            };
            self.expect_simple(TokenKind::Colon)?;
            let field_type = self.expect_ident_token()?;
            let field_type_name = match &field_type.kind {
                TokenKind::Ident(value) => value.clone(),
                _ => unreachable!(),
            };
            let mut required = false;
            if self.peek_ident("required") {
                self.expect_keyword("required")?;
                required = true;
            }
            let semicolon = self.expect_simple(TokenKind::Semicolon)?;
            fields.push(SchemaFieldDecl {
                name: field_name,
                field_type: field_type_name,
                required,
                span: SpanRange::new(
                    start.span.line,
                    start.span.column,
                    semicolon.end_line,
                    semicolon.end_column,
                ),
            });
        }
        self.expect_simple(TokenKind::RBrace)?;
        Ok(fields)
    }

    fn parse_verification_block(&mut self) -> Result<VerificationDecl, ValidateReport> {
        let start = self.expect_simple(TokenKind::LBrace)?;
        let mut verification = VerificationDecl {
            commands: Vec::new(),
            flows: Vec::new(),
            span: Some(start.clone()),
        };
        while !self.check_simple(TokenKind::RBrace) {
            let mut is_entry = false;
            if self.peek_ident("entry") {
                self.expect_keyword("entry")?;
                is_entry = true;
            }
            if self.peek_ident("command") {
                self.expect_keyword("command")?;
                verification.commands.push(self.parse_string_stmt()?);
                continue;
            }
            if self.peek_ident("flow") {
                self.expect_keyword("flow")?;
                verification.flows.push(self.parse_flow(is_entry)?);
                continue;
            }

            let token = self.expect_ident_token()?;
            let name = match &token.kind {
                TokenKind::Ident(value) => value.clone(),
                _ => unreachable!(),
            };
            return Err(self.syntax_error(
                token.span,
                format!("unknown verification item: {name}"),
                Some("expected `command` or `flow`".to_string()),
            ));
        }
        let end = self.expect_simple(TokenKind::RBrace)?;
        verification.span = Some(SpanRange::new(
            start.line,
            start.column,
            end.end_line,
            end.end_column,
        ));
        Ok(verification)
    }

    fn parse_guard_clause(&mut self) -> Result<GuardClause, ValidateReport> {
        let start = self.expect_keyword("guard")?;
        let condition_start = self.expect_string_token()?;
        let condition = match condition_start.kind {
            TokenKind::String(value) => value,
            _ => unreachable!(),
        };
        let end = self.expect_simple(TokenKind::Semicolon)?;
        Ok(GuardClause {
            condition,
            span: SpanRange::new(start.line, start.column, end.end_line, end.end_column),
        })
    }

    fn parse_flow(&mut self, is_entry: bool) -> Result<FlowDecl, ValidateReport> {
        let id = self.expect_ident_token()?;
        let flow_id = match &id.kind {
            TokenKind::Ident(value) => value.clone(),
            _ => unreachable!(),
        };
        let start = id.span.clone();
        self.expect_simple(TokenKind::LBrace)?;
        let mut steps = Vec::new();
        let mut binds = Vec::new();
        let mut recover = None;

        while !self.check_simple(TokenKind::RBrace) {
            if self.peek_ident("step") {
                self.expect_keyword("step")?;
                let step_start = self.expect_ident_token()?;
                let step_id = match &step_start.kind {
                    TokenKind::Ident(value) => value.clone(),
                    _ => unreachable!(),
                };
                self.expect_simple(TokenKind::Equals)?;
                let block = self.parse_dotted_ident()?;
                let mut guard = None;
                if self.peek_ident("guard") {
                    guard = Some(self.parse_guard_clause()?);
                }
                let end = self.expect_simple(TokenKind::Semicolon)?;
                steps.push(StepDecl {
                    id: step_id,
                    block: block.value,
                    guard,
                    span: SpanRange::new(
                        step_start.span.line,
                        step_start.span.column,
                        end.end_line,
                        end.end_column,
                    ),
                });
                continue;
            }

            if self.peek_ident("bind") {
                let bind_token = self.expect_keyword("bind")?;
                let from = self.parse_dotted_ident()?;
                self.expect_simple(TokenKind::Arrow)?;
                let to = self.parse_dotted_ident()?;
                let end = self.expect_simple(TokenKind::Semicolon)?;
                binds.push(BindDecl {
                    from: from.value,
                    to: to.value,
                    span: SpanRange::new(
                        bind_token.line,
                        bind_token.column,
                        end.end_line,
                        end.end_column,
                    ),
                });
                continue;
            }

            if self.peek_ident("recover") {
                if recover.is_some() {
                    let token = self.expect_ident_token()?;
                    return Err(self.syntax_error(
                        token.span,
                        "duplicate recover clause in flow".to_string(),
                        Some("only one recover clause is allowed per flow".to_string()),
                    ));
                }
                recover = Some(self.parse_recover_clause()?);
                continue;
            }

            let token = self.expect_ident_token()?;
            let name = match &token.kind {
                TokenKind::Ident(value) => value.clone(),
                _ => unreachable!(),
            };
            return Err(self.syntax_error(
                token.span,
                format!("unknown flow item: {name}"),
                Some("expected `step`, `bind`, or `recover`".to_string()),
            ));
        }
        let end = self.expect_simple(TokenKind::RBrace)?;
        Ok(FlowDecl {
            id: flow_id,
            span: SpanRange::new(start.line, start.column, end.end_line, end.end_column),
            is_entry,
            steps,
            binds,
            recover,
        })
    }

    fn parse_recover_clause(&mut self) -> Result<RecoverClause, ValidateReport> {
        let start = self.expect_keyword("recover")?;
        self.expect_simple(TokenKind::LBrace)?;
        let mut steps = Vec::new();
        let mut binds = Vec::new();

        while !self.check_simple(TokenKind::RBrace) {
            if self.peek_ident("step") {
                self.expect_keyword("step")?;
                let step_start = self.expect_ident_token()?;
                let step_id = match &step_start.kind {
                    TokenKind::Ident(value) => value.clone(),
                    _ => unreachable!(),
                };
                self.expect_simple(TokenKind::Equals)?;
                let block = self.parse_dotted_ident()?;
                let mut guard = None;
                if self.peek_ident("guard") {
                    guard = Some(self.parse_guard_clause()?);
                }
                let end = self.expect_simple(TokenKind::Semicolon)?;
                steps.push(StepDecl {
                    id: step_id,
                    block: block.value,
                    guard,
                    span: SpanRange::new(
                        step_start.span.line,
                        step_start.span.column,
                        end.end_line,
                        end.end_column,
                    ),
                });
                continue;
            }

            if self.peek_ident("bind") {
                let bind_token = self.expect_keyword("bind")?;
                let from = self.parse_dotted_ident()?;
                self.expect_simple(TokenKind::Arrow)?;
                let to = self.parse_dotted_ident()?;
                let end = self.expect_simple(TokenKind::Semicolon)?;
                binds.push(BindDecl {
                    from: from.value,
                    to: to.value,
                    span: SpanRange::new(
                        bind_token.line,
                        bind_token.column,
                        end.end_line,
                        end.end_column,
                    ),
                });
                continue;
            }

            let token = self.expect_ident_token()?;
            let name = match &token.kind {
                TokenKind::Ident(value) => value.clone(),
                _ => unreachable!(),
            };
            return Err(self.syntax_error(
                token.span,
                format!("unknown recover item: {name}"),
                Some("expected `step` or `bind`".to_string()),
            ));
        }

        let end = self.expect_simple(TokenKind::RBrace)?;
        Ok(RecoverClause {
            steps,
            binds,
            span: SpanRange::new(start.line, start.column, end.end_line, end.end_column),
        })
    }

    fn parse_dotted_ident(&mut self) -> Result<SpannedIdent, ValidateReport> {
        let first = self.expect_ident_token()?;
        let start = first.span.clone();
        let mut parts = vec![match first.kind {
            TokenKind::Ident(value) => value,
            _ => unreachable!(),
        }];
        let mut end = start.clone();
        while self.check_simple(TokenKind::Dot) {
            self.expect_simple(TokenKind::Dot)?;
            let token = self.expect_ident_token()?;
            end = token.span.clone();
            parts.push(match token.kind {
                TokenKind::Ident(value) => value,
                _ => unreachable!(),
            });
        }
        Ok(SpannedIdent {
            value: parts.join("."),
            span: SpanRange::new(start.line, start.column, end.end_line, end.end_column),
        })
    }

    fn expect_ident_stmt_in_block(&mut self) -> Result<SpannedIdent, ValidateReport> {
        let token = self.expect_ident_token()?;
        let end = self.expect_simple(TokenKind::Semicolon)?;
        match token.kind {
            TokenKind::Ident(value) => Ok(SpannedIdent {
                value,
                span: SpanRange::new(
                    token.span.line,
                    token.span.column,
                    end.end_line,
                    end.end_column,
                ),
            }),
            _ => unreachable!(),
        }
    }

    fn expect_keyword(&mut self, expected: &str) -> Result<SpanRange, ValidateReport> {
        let token = self.expect_ident_token()?;
        match token.kind {
            TokenKind::Ident(value) if value == expected => Ok(token.span),
            TokenKind::Ident(value) => Err(self.syntax_error(
                token.span,
                format!("expected keyword `{expected}`, got `{value}`"),
                None,
            )),
            _ => unreachable!(),
        }
    }

    fn expect_ident_token(&mut self) -> Result<Token, ValidateReport> {
        let Some(token) = self.tokens.get(self.index).cloned() else {
            return Err(self.syntax_error(
                SpanRange::new(1, 1, 1, 1),
                "unexpected end of file".to_string(),
                None,
            ));
        };
        self.index += 1;
        match token.kind {
            TokenKind::Ident(_) => Ok(token),
            _ => Err(self.syntax_error(token.span, "expected identifier".to_string(), None)),
        }
    }

    fn expect_string_token(&mut self) -> Result<Token, ValidateReport> {
        let Some(token) = self.tokens.get(self.index).cloned() else {
            return Err(self.syntax_error(
                SpanRange::new(1, 1, 1, 1),
                "unexpected end of file".to_string(),
                None,
            ));
        };
        self.index += 1;
        match token.kind {
            TokenKind::String(_) => Ok(token),
            _ => Err(self.syntax_error(token.span, "expected string literal".to_string(), None)),
        }
    }

    fn expect_simple(&mut self, expected: TokenKind) -> Result<SpanRange, ValidateReport> {
        let Some(token) = self.tokens.get(self.index).cloned() else {
            return Err(self.syntax_error(
                SpanRange::new(1, 1, 1, 1),
                "unexpected end of file".to_string(),
                None,
            ));
        };
        self.index += 1;
        if token.kind == expected {
            Ok(token.span)
        } else {
            Err(self.syntax_error(
                token.span,
                format!(
                    "unexpected token while expecting `{}`",
                    display_token(&expected)
                ),
                None,
            ))
        }
    }

    fn check_simple(&self, expected: TokenKind) -> bool {
        self.tokens
            .get(self.index)
            .is_some_and(|token| token.kind == expected)
    }

    fn peek_ident(&self, expected: &str) -> bool {
        self.tokens
            .get(self.index)
            .is_some_and(|token| match &token.kind {
                TokenKind::Ident(value) => value == expected,
                _ => false,
            })
    }

    fn unsupported_error(&self, span: SpanRange, message: String) -> ValidateReport {
        ValidateReport::error(
            self.source_path,
            RuleResult {
                error_id: "bcl.syntax.unsupported_construct".to_string(),
                rule_id: "BCL-SYNTAX-UNSUPPORTED-001".to_string(),
                severity: "error".to_string(),
                message,
                hint: Some(
                    "remove the unsupported construct or defer it to a later BCL phase".to_string(),
                ),
                span,
            },
        )
    }

    fn syntax_error(
        &self,
        span: SpanRange,
        message: String,
        hint: Option<String>,
    ) -> ValidateReport {
        ValidateReport::error(
            self.source_path,
            RuleResult {
                error_id: "bcl.syntax.parse_error".to_string(),
                rule_id: "BCL-SYNTAX-001".to_string(),
                severity: "error".to_string(),
                message,
                hint,
                span,
            },
        )
    }
}

fn display_token(token: &TokenKind) -> &'static str {
    match token {
        TokenKind::LBrace => "{",
        TokenKind::RBrace => "}",
        TokenKind::LParen => "(",
        TokenKind::RParen => ")",
        TokenKind::Equals => "=",
        TokenKind::Colon => ":",
        TokenKind::Semicolon => ";",
        TokenKind::Dot => ".",
        TokenKind::Arrow => "->",
        TokenKind::Ident(_) => "identifier",
        TokenKind::String(_) => "string",
    }
}

fn tokenize(source_path: &str, source: &str) -> Result<Vec<Token>, ValidateReport> {
    let mut chars = source.chars().peekable();
    let mut tokens = Vec::new();
    let mut line = 1usize;
    let mut column = 1usize;

    while let Some(ch) = chars.peek().copied() {
        match ch {
            ' ' | '\t' | '\r' => {
                chars.next();
                column += 1;
            }
            '\n' => {
                chars.next();
                line += 1;
                column = 1;
            }
            '{' => {
                tokens.push(simple_token(
                    TokenKind::LBrace,
                    line,
                    column,
                    line,
                    column + 1,
                ));
                chars.next();
                column += 1;
            }
            '}' => {
                tokens.push(simple_token(
                    TokenKind::RBrace,
                    line,
                    column,
                    line,
                    column + 1,
                ));
                chars.next();
                column += 1;
            }
            '(' => {
                tokens.push(simple_token(
                    TokenKind::LParen,
                    line,
                    column,
                    line,
                    column + 1,
                ));
                chars.next();
                column += 1;
            }
            ')' => {
                tokens.push(simple_token(
                    TokenKind::RParen,
                    line,
                    column,
                    line,
                    column + 1,
                ));
                chars.next();
                column += 1;
            }
            '=' => {
                tokens.push(simple_token(
                    TokenKind::Equals,
                    line,
                    column,
                    line,
                    column + 1,
                ));
                chars.next();
                column += 1;
            }
            ':' => {
                tokens.push(simple_token(
                    TokenKind::Colon,
                    line,
                    column,
                    line,
                    column + 1,
                ));
                chars.next();
                column += 1;
            }
            ';' => {
                tokens.push(simple_token(
                    TokenKind::Semicolon,
                    line,
                    column,
                    line,
                    column + 1,
                ));
                chars.next();
                column += 1;
            }
            '.' => {
                tokens.push(simple_token(TokenKind::Dot, line, column, line, column + 1));
                chars.next();
                column += 1;
            }
            '-' => {
                let start_column = column;
                chars.next();
                column += 1;
                if chars.peek() == Some(&'>') {
                    chars.next();
                    column += 1;
                    tokens.push(simple_token(
                        TokenKind::Arrow,
                        line,
                        start_column,
                        line,
                        column,
                    ));
                } else {
                    return Err(tokenize_error(
                        source_path,
                        SpanRange::new(line, start_column, line, column),
                        "unexpected `-`".to_string(),
                    ));
                }
            }
            '"' => {
                let start_line = line;
                let start_column = column;
                chars.next();
                column += 1;
                let mut value = String::new();
                let mut terminated = false;
                while let Some(next) = chars.next() {
                    match next {
                        '"' => {
                            column += 1;
                            terminated = true;
                            break;
                        }
                        '\n' => {
                            value.push(next);
                            line += 1;
                            column = 1;
                        }
                        _ => {
                            value.push(next);
                            column += 1;
                        }
                    }
                }
                if !terminated {
                    return Err(tokenize_error(
                        source_path,
                        SpanRange::new(start_line, start_column, line, column),
                        "unterminated string literal".to_string(),
                    ));
                }
                tokens.push(simple_token(
                    TokenKind::String(value),
                    start_line,
                    start_column,
                    line,
                    column,
                ));
            }
            _ if is_ident_start(ch) => {
                let start_line = line;
                let start_column = column;
                let value = read_ident(&mut chars, &mut column);
                tokens.push(simple_token(
                    TokenKind::Ident(value),
                    start_line,
                    start_column,
                    line,
                    column,
                ));
            }
            _ => {
                let span = SpanRange::new(line, column, line, column + 1);
                return Err(tokenize_error(
                    source_path,
                    span,
                    format!("unexpected character `{ch}`"),
                ));
            }
        }
    }

    Ok(tokens)
}

fn read_ident(chars: &mut Peekable<Chars<'_>>, column: &mut usize) -> String {
    let mut value = String::new();
    while let Some(ch) = chars.peek().copied() {
        if is_ident_continue(ch) {
            value.push(ch);
            chars.next();
            *column += 1;
        } else {
            break;
        }
    }
    value
}

fn is_ident_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

fn is_ident_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '-'
}

fn simple_token(
    kind: TokenKind,
    line: usize,
    column: usize,
    end_line: usize,
    end_column: usize,
) -> Token {
    Token {
        kind,
        span: SpanRange::new(line, column, end_line, end_column),
    }
}

fn tokenize_error(source: &str, span: SpanRange, message: String) -> ValidateReport {
    ValidateReport::error(
        source,
        RuleResult {
            error_id: "bcl.syntax.parse_error".to_string(),
            rule_id: "BCL-SYNTAX-001".to_string(),
            severity: "error".to_string(),
            message,
            hint: None,
            span,
        },
    )
}
