use crate::lexer::lex_tokens;
use crate::types::{
    AdtCtorExpr, AdtDecl, AdtMatchPattern, AdtPatternItem, AdtVariant, AstArena, BinaryOp,
    BlockExpr, CallArg, CaptureMode, ClosureCapturePolicy, ClosureLiteral, ClosureValueFamily,
    ExecutableImport, ExecutableImportSelectItem, Expr, ExprId, FrontendError, Function, IfExpr,
    IfLetExpr, ImplDecl, IntRangePattern, IterableLoopDesugaring, LogosEntity, LogosEntityField,
    LogosEntityFieldKind, LogosLaw, LogosProgram, LogosSystem, LogosWhen, LoopExpr, MapType,
    MatchArm, MatchExpr, MatchExprArm, MatchPattern, NumericLiteral, Program, QuadVal, RangeExpr,
    RecordDecl, RecordField, RecordFieldExpr, RecordInitField, RecordLiteralExpr,
    RecordPatternItem, RecordPatternTarget, RecordUpdateExpr, SchemaDecl, SchemaField, SchemaRole,
    SchemaShape, SchemaVariant, SchemaVersion, SequenceCollectionFamily, SequenceIndexExpr,
    SequenceLiteral, SequenceType, Stmt, StmtId, SymbolId, TextLiteral, TextLiteralFamily, Token,
    TokenKind, TraitBound, TraitDecl, TraitMethodSig, TuplePatternItem, Type, UnaryOp,
};
use crate::CompilePolicyView;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use sm_profile::{CompatibilityMode, ParserProfile};
use ton618_core::diagnostics::{
    format_multiple_parser_errors, format_parser_error_at_input, suggest_closest_case_insensitive,
};
use ton618_core::SourceMap;

pub fn parse_rustlike_with_profile(
    input: &str,
    profile: &ParserProfile,
) -> Result<Program, FrontendError> {
    let tokens = lex_tokens(input)?;
    let mut p = Parser {
        tokens,
        idx: 0,
        source: input.to_string(),
        arena: AstArena::default(),
        policy: CompilePolicyView::new(profile),
        type_param_scope: Vec::new(),
        self_type_scope: None,
    };
    p.parse_program()
}

pub fn parse_logos_with_profile(
    input: &str,
    profile: &ParserProfile,
) -> Result<LogosProgram, FrontendError> {
    let tokens = lex_tokens(input)?;
    let mut p = Parser {
        tokens,
        idx: 0,
        source: input.to_string(),
        arena: AstArena::default(),
        policy: CompilePolicyView::new(profile),
        type_param_scope: Vec::new(),
        self_type_scope: None,
    };
    p.parse_logos_program()
}

struct Parser<'a> {
    tokens: Vec<Token>,
    idx: usize,
    source: String,
    arena: AstArena,
    policy: CompilePolicyView<'a>,
    /// Type parameters currently in scope for the declaration being parsed.
    ///
    /// Populated by `parse_type_params` and cleared after each
    /// function/record/adt declaration finishes parsing. Drives `parse_type`
    /// to emit `Type::TypeVar` rather than `Type::Record` for matching names.
    type_param_scope: Vec<SymbolId>,
    /// Narrow owner-layer `Self` marker available only while parsing trait
    /// method signatures or impl methods.
    self_type_scope: Option<Type>,
}

impl<'a> Parser<'a> {
    fn with_self_type_scope<T, F>(&mut self, self_ty: Type, f: F) -> Result<T, FrontendError>
    where
        F: FnOnce(&mut Self) -> Result<T, FrontendError>,
    {
        let previous = self.self_type_scope.clone();
        self.self_type_scope = Some(self_ty);
        let result = f(self);
        self.self_type_scope = previous;
        result
    }

    fn parse_program(&mut self) -> Result<Program, FrontendError> {
        let mut imports = Vec::new();
        let mut adts = Vec::new();
        let mut records = Vec::new();
        let mut schemas = Vec::new();
        let mut functions = Vec::new();
        let mut traits = Vec::new();
        let mut impls = Vec::new();
        loop {
            let i = self.next_non_layout_idx();
            if i >= self.tokens.len() {
                break;
            }
            self.idx = i;
            if self.starts_role_marked_schema_decl() {
                schemas.push(self.parse_schema_decl()?);
                continue;
            }
            match self.tokens[i].kind {
                TokenKind::KwImport => imports.push(self.parse_import_decl()?),
                TokenKind::KwEnum => adts.push(self.parse_adt_decl()?),
                TokenKind::KwFn => functions.push(self.parse_function()?),
                TokenKind::KwRecord => records.push(self.parse_record_decl()?),
                TokenKind::KwSchema => schemas.push(self.parse_schema_decl()?),
                TokenKind::KwTrait => traits.push(self.parse_trait_decl()?),
                TokenKind::KwImpl => impls.push(self.parse_impl_decl()?),
                _ => {
                    return Err(FrontendError {
                        pos: self.tokens[i].pos,
                        message:
                            "expected top-level 'Import', 'enum', 'fn', 'impl', 'record', 'schema', 'trait', or role-marked schema declaration"
                                .to_string(),
                    });
                }
            }
        }
        Ok(Program {
            arena: ::core::mem::take(&mut self.arena),
            imports,
            adts,
            records,
            schemas,
            functions,
            traits,
            impls,
        })
    }

    fn parse_import_decl(&mut self) -> Result<ExecutableImport, FrontendError> {
        self.expect(TokenKind::KwImport, "expected 'Import'")?;
        let reexport = self.eat_ident_text("pub");
        let spec = self.expect_string_literal_text("expected import path string")?;
        let alias = if self.eat_ident_text("as") {
            Some(self.expect_symbol()?)
        } else {
            None
        };
        let wildcard = self.eat(TokenKind::Star);
        let mut select_items = Vec::new();
        if self.eat(TokenKind::LBrace) {
            if !self.check(TokenKind::RBrace) {
                loop {
                    let name = self.expect_symbol()?;
                    let alias = if self.eat_ident_text("as") {
                        Some(self.expect_symbol()?)
                    } else {
                        None
                    };
                    select_items.push(ExecutableImportSelectItem { name, alias });
                    if self.eat(TokenKind::Comma) {
                        continue;
                    }
                    break;
                }
            }
            self.expect(TokenKind::RBrace, "expected '}' after import select list")?;
        }
        Ok(ExecutableImport {
            spec,
            alias,
            reexport,
            select_items,
            wildcard,
        })
    }

    fn parse_function(&mut self) -> Result<Function, FrontendError> {
        self.expect(TokenKind::KwFn, "expected 'fn'")?;
        let name = self.expect_symbol()?;
        let (type_params, trait_bounds) = self.parse_type_params_with_bounds()?;
        self.expect(TokenKind::LParen, "expected '('")?;
        let mut params = Vec::new();
        let mut param_defaults = Vec::new();
        let mut default_seen = false;
        if !self.check(TokenKind::RParen) {
            loop {
                let pname = self.expect_symbol()?;
                self.expect(TokenKind::Colon, "expected ':'")?;
                let pty = self.parse_type()?;
                let default = if self.eat(TokenKind::Assign) {
                    default_seen = true;
                    Some(self.parse_expr()?)
                } else {
                    if default_seen {
                        return Err(FrontendError {
                            pos: self.pos(),
                            message:
                                "required parameter cannot follow parameter with default value"
                                    .to_string(),
                        });
                    }
                    None
                };
                params.push((pname, pty));
                param_defaults.push(default);
                if self.eat(TokenKind::Comma) {
                    continue;
                }
                break;
            }
        }
        self.expect(TokenKind::RParen, "expected ')'")?;
        let ret = if self.eat(TokenKind::Implies) {
            self.parse_type()?
        } else {
            Type::Unit
        };
        let (requires, ensures, invariants) = self.parse_contract_clauses()?;
        let body = if self.eat(TokenKind::Assign) {
            let expr = self.parse_expr()?;
            self.expect(
                TokenKind::Semi,
                "expected ';' after expression-bodied function",
            )?;
            vec![self.arena.alloc_stmt(Stmt::Return(Some(expr)))]
        } else {
            self.parse_block()?
        };
        self.pop_type_param_scope(type_params.len());
        Ok(Function {
            name,
            type_params,
            trait_bounds,
            params,
            param_defaults,
            requires,
            ensures,
            invariants,
            ret,
            body,
        })
    }

    fn parse_contract_clauses(
        &mut self,
    ) -> Result<(Vec<ExprId>, Vec<ExprId>, Vec<ExprId>), FrontendError> {
        let mut requires = Vec::new();
        while self.eat(TokenKind::KwRequires) {
            self.expect(TokenKind::LParen, "expected '(' after 'requires'")?;
            let condition = self.parse_expr()?;
            self.expect(TokenKind::RParen, "expected ')' after requires condition")?;
            requires.push(condition);
        }
        let mut ensures = Vec::new();
        while self.eat(TokenKind::KwEnsures) {
            self.expect(TokenKind::LParen, "expected '(' after 'ensures'")?;
            let condition = self.parse_expr()?;
            self.expect(TokenKind::RParen, "expected ')' after ensures condition")?;
            ensures.push(condition);
        }
        let mut invariants = Vec::new();
        while self.eat(TokenKind::KwInvariant) {
            self.expect(TokenKind::LParen, "expected '(' after 'invariant'")?;
            let condition = self.parse_expr()?;
            self.expect(TokenKind::RParen, "expected ')' after invariant condition")?;
            invariants.push(condition);
        }
        Ok((requires, ensures, invariants))
    }

    /// Parse an optional `<T, U: Bound, ...>` type parameter list with
    /// optional trait bounds.
    ///
    /// Returns `(type_params, trait_bounds)`. Type parameter names are pushed
    /// into `self.type_param_scope` so `parse_type` can emit `Type::TypeVar`
    /// for matching names. The caller must call `pop_type_param_scope(count)`
    /// after parsing the body.
    fn parse_type_params_with_bounds(
        &mut self,
    ) -> Result<(Vec<SymbolId>, Vec<TraitBound>), FrontendError> {
        if !self.eat(TokenKind::LAngle) {
            return Ok((Vec::new(), Vec::new()));
        }
        let mut params = Vec::new();
        let mut bounds = Vec::new();
        loop {
            if self.check(TokenKind::RAngle) {
                break;
            }
            let param_id = self.expect_type_param_name()?;
            params.push(param_id);
            self.type_param_scope.push(param_id);
            // Optional `: TraitName` bound on this parameter.
            if self.eat(TokenKind::Colon) {
                let bound_name = self.expect_symbol()?;
                bounds.push(TraitBound {
                    param: param_id,
                    bound: bound_name,
                });
            }
            if self.eat(TokenKind::Comma) {
                continue;
            }
            break;
        }
        if params.is_empty() {
            return Err(FrontendError {
                pos: self.pos(),
                message: "empty type parameter list is not allowed".to_string(),
            });
        }
        self.expect(TokenKind::RAngle, "expected '>' after type parameter list")?;
        Ok((params, bounds))
    }

    /// Parse an optional `<T, U, ...>` type parameter list (no bounds).
    ///
    /// Thin wrapper over `parse_type_params_with_bounds` that discards any
    /// bounds. Used for record and ADT declarations where trait bounds on type
    /// parameters are not admitted in first wave.
    fn parse_type_params(&mut self) -> Result<Vec<SymbolId>, FrontendError> {
        let (params, _bounds) = self.parse_type_params_with_bounds()?;
        Ok(params)
    }

    /// Remove `count` entries from the tail of `type_param_scope`.
    fn pop_type_param_scope(&mut self, count: usize) {
        let new_len = self.type_param_scope.len().saturating_sub(count);
        self.type_param_scope.truncate(new_len);
    }

    /// Parse a `trait TraitName { fn method(params) -> ret; ... }` declaration.
    fn parse_trait_decl(&mut self) -> Result<TraitDecl, FrontendError> {
        self.expect(TokenKind::KwTrait, "expected 'trait'")?;
        let name = self.expect_symbol()?;
        let type_params = self.parse_type_params()?;
        self.expect(TokenKind::LBrace, "expected '{' after trait name")?;
        let self_placeholder = Type::TypeVar(self.arena.intern_symbol("Self"));
        let methods = self.with_self_type_scope(self_placeholder, |parser| {
            let mut methods = Vec::new();
            loop {
                let i = parser.next_non_layout_idx();
                if i >= parser.tokens.len() {
                    break;
                }
                if parser.tokens[i].kind == TokenKind::RBrace {
                    parser.idx = i;
                    break;
                }
                parser.idx = i;
                methods.push(parser.parse_trait_method_sig()?);
            }
            Ok(methods)
        })?;
        self.expect(TokenKind::RBrace, "expected '}' to close trait body")?;
        self.pop_type_param_scope(type_params.len());
        Ok(TraitDecl {
            name,
            type_params,
            methods,
        })
    }

    /// Parse an abstract method signature inside a trait body:
    /// `fn name(param: Type, ...) -> RetType;`
    fn parse_trait_method_sig(&mut self) -> Result<TraitMethodSig, FrontendError> {
        self.expect(TokenKind::KwFn, "expected 'fn' for trait method signature")?;
        let name = self.expect_symbol()?;
        self.expect(TokenKind::LParen, "expected '(' after trait method name")?;
        let mut params = Vec::new();
        if !self.check(TokenKind::RParen) {
            loop {
                let pname = self.expect_symbol()?;
                self.expect(TokenKind::Colon, "expected ':' after parameter name")?;
                let pty = self.parse_type()?;
                params.push((pname, pty));
                if self.eat(TokenKind::Comma) {
                    continue;
                }
                break;
            }
        }
        self.expect(
            TokenKind::RParen,
            "expected ')' after trait method parameters",
        )?;
        let ret = if self.eat(TokenKind::Implies) {
            self.parse_type()?
        } else {
            Type::Unit
        };
        self.expect(TokenKind::Semi, "expected ';' after trait method signature")?;
        Ok(TraitMethodSig { name, params, ret })
    }

    /// Parse an `impl TraitName for TypeName { fn method(...) { ... } ... }` block.
    fn parse_impl_decl(&mut self) -> Result<ImplDecl, FrontendError> {
        self.expect(TokenKind::KwImpl, "expected 'impl'")?;
        let type_params = self.parse_type_params()?;
        let trait_name = self.expect_symbol()?;
        self.expect(TokenKind::KwFor, "expected 'for' after trait name in impl")?;
        let for_type = self.expect_symbol()?;
        self.expect(TokenKind::LBrace, "expected '{' after impl target type")?;
        let methods = self.with_self_type_scope(Type::Record(for_type), |parser| {
            let mut methods = Vec::new();
            loop {
                let i = parser.next_non_layout_idx();
                if i >= parser.tokens.len() {
                    break;
                }
                if parser.tokens[i].kind == TokenKind::RBrace {
                    parser.idx = i;
                    break;
                }
                parser.idx = i;
                methods.push(parser.parse_function()?);
            }
            Ok(methods)
        })?;
        self.expect(TokenKind::RBrace, "expected '}' to close impl body")?;
        self.pop_type_param_scope(type_params.len());
        Ok(ImplDecl {
            trait_name,
            for_type,
            type_params,
            methods,
        })
    }

    fn parse_record_decl(&mut self) -> Result<RecordDecl, FrontendError> {
        self.expect(TokenKind::KwRecord, "expected 'record'")?;
        let name = self.expect_symbol()?;
        let type_params = self.parse_type_params()?;
        self.expect(TokenKind::LBrace, "expected '{' after record name")?;
        let mut fields = Vec::new();
        while !self.check(TokenKind::RBrace) {
            let field_name = self.expect_symbol()?;
            self.expect(TokenKind::Colon, "expected ':' after record field name")?;
            let field_ty = self.parse_type()?;
            fields.push(RecordField {
                name: field_name,
                ty: field_ty,
            });
            if self.eat(TokenKind::Comma) {
                if self.check(TokenKind::RBrace) {
                    break;
                }
                continue;
            }
            break;
        }
        self.expect(TokenKind::RBrace, "expected '}' after record declaration")?;
        self.pop_type_param_scope(type_params.len());
        Ok(RecordDecl {
            name,
            type_params,
            fields,
        })
    }

    fn parse_schema_decl(&mut self) -> Result<SchemaDecl, FrontendError> {
        self.require_schema_surface("schema declarations are disabled by profile policy")?;
        let role = self.parse_optional_schema_role()?;
        self.expect(TokenKind::KwSchema, "expected 'schema'")?;
        let name = self.expect_symbol()?;
        let version = self.parse_optional_schema_version()?;
        self.expect(TokenKind::LBrace, "expected '{' after schema name")?;
        let shape = if self.check(TokenKind::RBrace) {
            SchemaShape::Record(Vec::new())
        } else {
            let first_name = self.expect_symbol()?;
            if self.check(TokenKind::Colon) {
                SchemaShape::Record(self.parse_schema_record_fields_after_first(first_name)?)
            } else if self.check(TokenKind::LBrace) {
                SchemaShape::TaggedUnion(
                    self.parse_schema_tagged_union_variants_after_first(first_name)?,
                )
            } else {
                return Err(FrontendError {
                    pos: self.pos(),
                    message:
                        "schema declaration body must use either 'field: type' entries or 'Variant { ... }' entries"
                            .to_string(),
                });
            }
        };
        self.expect(TokenKind::RBrace, "expected '}' after schema declaration")?;
        Ok(SchemaDecl {
            name,
            role,
            version,
            shape,
        })
    }

    fn starts_role_marked_schema_decl(&self) -> bool {
        let i = self.next_non_layout_idx();
        let Some(first) = self.tokens.get(i) else {
            return false;
        };
        if first.kind != TokenKind::Ident || !Self::is_schema_role_marker_text(&first.text) {
            return false;
        }
        let next = self.next_non_layout_idx_from(i + 1);
        self.tokens
            .get(next)
            .map(|tok| tok.kind == TokenKind::KwSchema)
            .unwrap_or(false)
    }

    fn parse_optional_schema_role(&mut self) -> Result<Option<SchemaRole>, FrontendError> {
        if !self.starts_role_marked_schema_decl() {
            return Ok(None);
        }
        let role_tok = self.advance();
        let role = match role_tok.text.as_str() {
            "config" => SchemaRole::Config,
            "api" => SchemaRole::Api,
            "wire" => SchemaRole::Wire,
            _ => {
                return Err(FrontendError {
                    pos: role_tok.pos,
                    message: "unknown schema role marker".to_string(),
                })
            }
        };
        Ok(Some(role))
    }

    fn is_schema_role_marker_text(text: &str) -> bool {
        matches!(text, "config" | "api" | "wire")
    }

    fn starts_schema_version_marker(&self) -> bool {
        let i = self.next_non_layout_idx();
        let Some(first) = self.tokens.get(i) else {
            return false;
        };
        if first.kind != TokenKind::Ident || first.text != "version" {
            return false;
        }
        let next = self.next_non_layout_idx_from(i + 1);
        self.tokens
            .get(next)
            .map(|tok| tok.kind == TokenKind::LParen)
            .unwrap_or(false)
    }

    fn parse_optional_schema_version(&mut self) -> Result<Option<SchemaVersion>, FrontendError> {
        if !self.starts_schema_version_marker() {
            return Ok(None);
        }
        let marker = self.advance();
        debug_assert_eq!(marker.text, "version");
        self.expect(
            TokenKind::LParen,
            "expected '(' after schema version marker",
        )?;
        if !self.check(TokenKind::Num) {
            return Err(FrontendError {
                pos: self.pos(),
                message: "schema version marker currently requires unsuffixed decimal integer"
                    .to_string(),
            });
        }
        let number = self.advance();
        let value =
            parse_schema_version_literal(&number.text).map_err(|message| FrontendError {
                pos: number.pos,
                message,
            })?;
        self.expect(
            TokenKind::RParen,
            "expected ')' after schema version marker",
        )?;
        Ok(Some(SchemaVersion { value }))
    }

    fn parse_schema_record_fields_after_first(
        &mut self,
        first_name: SymbolId,
    ) -> Result<Vec<SchemaField>, FrontendError> {
        let mut fields = Vec::new();
        let mut field_name = first_name;
        loop {
            self.expect(TokenKind::Colon, "expected ':' after schema field name")?;
            let field_ty = self.parse_type()?;
            fields.push(SchemaField {
                name: field_name,
                ty: field_ty,
            });
            if self.eat(TokenKind::Comma) {
                if self.check(TokenKind::RBrace) {
                    break;
                }
                field_name = self.expect_symbol()?;
                if !self.check(TokenKind::Colon) {
                    return Err(FrontendError {
                        pos: self.pos(),
                        message:
                            "record-shaped schema declarations cannot mix field entries with tagged-union variants"
                                .to_string(),
                    });
                }
                continue;
            }
            break;
        }
        Ok(fields)
    }

    fn parse_schema_tagged_union_variants_after_first(
        &mut self,
        first_name: SymbolId,
    ) -> Result<Vec<SchemaVariant>, FrontendError> {
        let mut variants = Vec::new();
        let mut variant_name = first_name;
        loop {
            self.expect(
                TokenKind::LBrace,
                "expected '{' after schema variant name in tagged-union schema",
            )?;
            let mut fields = Vec::new();
            while !self.check(TokenKind::RBrace) {
                let field_name = self.expect_symbol()?;
                self.expect(TokenKind::Colon, "expected ':' after schema field name")?;
                let field_ty = self.parse_type()?;
                fields.push(SchemaField {
                    name: field_name,
                    ty: field_ty,
                });
                if self.eat(TokenKind::Comma) {
                    if self.check(TokenKind::RBrace) {
                        break;
                    }
                    continue;
                }
                break;
            }
            self.expect(
                TokenKind::RBrace,
                "expected '}' after schema variant payload",
            )?;
            variants.push(SchemaVariant {
                name: variant_name,
                fields,
            });
            if self.eat(TokenKind::Comma) {
                if self.check(TokenKind::RBrace) {
                    break;
                }
                variant_name = self.expect_symbol()?;
                if !self.check(TokenKind::LBrace) {
                    return Err(FrontendError {
                        pos: self.pos(),
                        message:
                            "tagged-union schema declarations cannot mix variant entries with record-shaped fields"
                                .to_string(),
                    });
                }
                continue;
            }
            break;
        }
        Ok(variants)
    }

    fn parse_adt_decl(&mut self) -> Result<AdtDecl, FrontendError> {
        self.expect(TokenKind::KwEnum, "expected 'enum'")?;
        let name = self.expect_symbol()?;
        let type_params = self.parse_type_params()?;
        self.expect(TokenKind::LBrace, "expected '{' after enum name")?;
        let mut variants = Vec::new();
        while !self.check(TokenKind::RBrace) {
            let variant_name = self.expect_symbol()?;
            let payload = if self.eat(TokenKind::LParen) {
                self.parse_adt_variant_payload_types()?
            } else {
                Vec::new()
            };
            variants.push(AdtVariant {
                name: variant_name,
                payload,
            });
            if self.eat(TokenKind::Comma) {
                if self.check(TokenKind::RBrace) {
                    break;
                }
                continue;
            }
            break;
        }
        self.expect(TokenKind::RBrace, "expected '}' after enum declaration")?;
        self.pop_type_param_scope(type_params.len());
        Ok(AdtDecl {
            name,
            type_params,
            variants,
        })
    }

    fn parse_adt_variant_payload_types(&mut self) -> Result<Vec<Type>, FrontendError> {
        let mut payload = Vec::new();
        if self.check(TokenKind::RParen) {
            return Err(FrontendError {
                pos: self.pos(),
                message: "enum constructor payload cannot be empty parentheses; omit '()' for unit variant".to_string(),
            });
        }
        loop {
            payload.push(self.parse_type()?);
            if self.eat(TokenKind::Comma) {
                if self.check(TokenKind::RParen) {
                    break;
                }
                continue;
            }
            break;
        }
        self.expect(TokenKind::RParen, "expected ')' after enum variant payload")?;
        Ok(payload)
    }

    fn parse_block(&mut self) -> Result<Vec<StmtId>, FrontendError> {
        self.expect(TokenKind::LBrace, "expected '{'")?;
        let mut out = Vec::new();
        while !self.check(TokenKind::RBrace) {
            out.push(self.parse_stmt()?);
        }
        self.expect(TokenKind::RBrace, "expected '}'")?;
        Ok(out)
    }

    fn parse_stmt(&mut self) -> Result<StmtId, FrontendError> {
        if self.eat(TokenKind::KwConst) {
            let name = self.expect_symbol()?;
            let ty = if self.eat(TokenKind::Colon) {
                Some(self.parse_type()?)
            } else {
                None
            };
            self.expect(TokenKind::Assign, "expected '='")?;
            let value = self.parse_expr()?;
            self.expect(TokenKind::Semi, "expected ';'")?;
            return Ok(self.arena.alloc_stmt(Stmt::Const { name, ty, value }));
        }
        if self.eat(TokenKind::KwLet) {
            let let_is_mut = self.eat_ident_text("mut");
            if self.eat(TokenKind::Underscore) {
                if let_is_mut {
                    return Err(FrontendError {
                        pos: self.pos(),
                        message: "let mut currently requires a plain binding target".to_string(),
                    });
                }
                let ty = if self.eat(TokenKind::Colon) {
                    Some(self.parse_type()?)
                } else {
                    None
                };
                self.expect(TokenKind::Assign, "expected '='")?;
                let value = self.parse_expr()?;
                if self.check(TokenKind::KwElse) {
                    return Err(FrontendError {
                        pos: self.pos(),
                        message:
                            "let-else currently requires tuple destructuring target; discard target is not supported"
                                .to_string(),
                    });
                }
                self.expect(TokenKind::Semi, "expected ';'")?;
                return Ok(self.arena.alloc_stmt(Stmt::Discard { ty, value }));
            }
            if self.eat(TokenKind::LParen) {
                if let_is_mut {
                    return Err(FrontendError {
                        pos: self.pos(),
                        message: "let mut currently requires a plain binding target".to_string(),
                    });
                }
                let items = self.parse_tuple_pattern_items_after_lparen()?;
                let ty = if self.eat(TokenKind::Colon) {
                    Some(self.parse_type()?)
                } else {
                    None
                };
                self.expect(TokenKind::Assign, "expected '='")?;
                let value = self.parse_expr()?;
                if self.eat(TokenKind::KwElse) {
                    let else_return = self.parse_else_return_payload("let-else")?;
                    return Ok(self.arena.alloc_stmt(Stmt::LetElseTuple {
                        items,
                        ty,
                        value,
                        else_return,
                    }));
                }
                // M9.4 Wave 3: if any item is Nested, emit LetElseTuple (no else arm)
                // so the typecheck path can handle recursive binding.
                // Note: `=` and `value` are already consumed above.
                let has_nested = items
                    .iter()
                    .any(|i| matches!(i, TuplePatternItem::Nested(_)));
                if has_nested {
                    self.expect(TokenKind::Semi, "expected ';'")?;
                    return Ok(self.arena.alloc_stmt(Stmt::LetElseTuple {
                        items,
                        ty,
                        value,
                        else_return: None,
                    }));
                }
                if items
                    .iter()
                    .any(|item| matches!(item, TuplePatternItem::QuadLiteral(_)))
                {
                    return Err(FrontendError {
                        pos: self.pos(),
                        message:
                            "quad literal tuple patterns currently require let-else; plain tuple destructuring bind supports only name/_/ref items"
                                .to_string(),
                    });
                }
                self.expect(TokenKind::Semi, "expected ';'")?;
                return Ok(self.arena.alloc_stmt(Stmt::LetTuple { items, ty, value }));
            }
            let name = self.expect_symbol()?;
            if self.check(TokenKind::LBrace) {
                if let_is_mut {
                    return Err(FrontendError {
                        pos: self.pos(),
                        message: "let mut currently requires a plain binding target".to_string(),
                    });
                }
                self.expect(TokenKind::LBrace, "expected '{' after record pattern name")?;
                let items = self.parse_record_pattern_items_after_lbrace()?;
                self.expect(TokenKind::Assign, "expected '='")?;
                let value = self.parse_expr()?;
                if self.eat(TokenKind::KwElse) {
                    let else_return = self.parse_else_return_payload("record let-else")?;
                    return Ok(self.arena.alloc_stmt(Stmt::LetElseRecord {
                        record_name: name,
                        items,
                        value,
                        else_return,
                    }));
                }
                let items = self.lower_record_pattern_items_to_bind(items)?;
                self.expect(TokenKind::Semi, "expected ';'")?;
                return Ok(self.arena.alloc_stmt(Stmt::LetRecord {
                    record_name: name,
                    items,
                    value,
                }));
            }
            let ty = if self.eat(TokenKind::Colon) {
                Some(self.parse_type()?)
            } else {
                None
            };
            self.expect(TokenKind::Assign, "expected '='")?;
            let value = self.parse_expr()?;
            if self.check(TokenKind::KwElse) {
                return Err(FrontendError {
                    pos: self.pos(),
                    message:
                        "let-else currently requires tuple destructuring target; plain binding target is not supported"
                            .to_string(),
                });
            }
            self.expect(TokenKind::Semi, "expected ';'")?;
            return Ok(self.arena.alloc_stmt(Stmt::Let {
                name,
                is_mut: let_is_mut,
                ty,
                value,
            }));
        }
        if self.check(TokenKind::Ident) {
            if let Some(op) = self.peek_compound_assign_op() {
                let name = self.expect_symbol()?;
                let _ = self.advance();
                let rhs = self.parse_expr()?;
                self.expect(TokenKind::Semi, "expected ';'")?;
                let lhs = self.arena.alloc_expr(Expr::Var(name));
                let value = self.arena.alloc_expr(Expr::Binary(lhs, op, rhs));
                return Ok(self.arena.alloc_stmt(Stmt::Assign { name, value }));
            }
            if self.peek_next_non_layout_kind() == Some(TokenKind::Assign) {
                let name = self.expect_symbol()?;
                self.expect(TokenKind::Assign, "expected '='")?;
                let value = self.parse_expr()?;
                self.expect(TokenKind::Semi, "expected ';'")?;
                return Ok(self.arena.alloc_stmt(Stmt::Assign { name, value }));
            }
        }
        if self.looks_like_tuple_assign_stmt() {
            self.expect(TokenKind::LParen, "expected '('")?;
            let items = self.parse_tuple_pattern_items_after_lparen()?;
            let items = self.lower_tuple_pattern_items_to_bind(items)?;
            self.expect(TokenKind::Assign, "expected '='")?;
            let value = self.parse_expr()?;
            self.expect(TokenKind::Semi, "expected ';'")?;
            return Ok(self.arena.alloc_stmt(Stmt::AssignTuple { items, value }));
        }
        if self.eat(TokenKind::KwFor) {
            let name = self.expect_symbol()?;
            self.expect(TokenKind::KwIn, "expected 'in' after for binding")?;
            let iterable = self.parse_expr()?;
            let body = self.parse_block()?;
            let iterable_trait = self.arena.intern_symbol("Iterable");
            return Ok(match self.arena.expr(iterable) {
                Expr::Range(_) => self.arena.alloc_stmt(Stmt::ForRange {
                    name,
                    range: iterable,
                    body,
                }),
                _ => self.arena.alloc_stmt(Stmt::ForEach {
                    name,
                    iterable,
                    body,
                    desugaring: IterableLoopDesugaring {
                        trait_name: iterable_trait,
                    },
                }),
            });
        }
        if self.eat(TokenKind::KwWhile) {
            let condition = self.parse_expr()?;
            let body = self.parse_block()?;
            return Ok(self.arena.alloc_stmt(Stmt::While { condition, body }));
        }
        if self.eat(TokenKind::KwLoop) {
            let body = self.parse_block()?;
            return Ok(self.arena.alloc_stmt(Stmt::Loop { body }));
        }
        if self.eat(TokenKind::KwGuard) {
            let condition = self.parse_expr()?;
            if !self.eat(TokenKind::KwElse) {
                return Err(FrontendError {
                    pos: self.pos(),
                    message: "guard clause requires else return branch".to_string(),
                });
            }
            if !self.eat(TokenKind::KwReturn) {
                return Err(FrontendError {
                    pos: self.pos(),
                    message: "guard clause currently supports only else return".to_string(),
                });
            }
            let else_return = if self.eat(TokenKind::Semi) {
                None
            } else {
                let expr = self.parse_expr()?;
                self.expect(TokenKind::Semi, "expected ';'")?;
                Some(expr)
            };
            return Ok(self.arena.alloc_stmt(Stmt::Guard {
                condition,
                else_return,
            }));
        }
        if self.eat(TokenKind::KwIf) {
            let condition = self.parse_expr()?;
            let then_block = self.parse_block()?;
            let else_block = if self.eat(TokenKind::KwElse) {
                if self.eat(TokenKind::KwIf) {
                    let nested = self.parse_if_after_kw_if()?;
                    vec![self.arena.alloc_stmt(nested)]
                } else {
                    self.parse_block()?
                }
            } else {
                Vec::new()
            };
            return Ok(self.arena.alloc_stmt(Stmt::If {
                condition,
                then_block,
                else_block,
            }));
        }
        if self.eat(TokenKind::KwMatch) {
            return self.parse_match_stmt_after_kw_match();
        }
        if self.eat(TokenKind::KwReturn) {
            if self.eat(TokenKind::Semi) {
                return Ok(self.arena.alloc_stmt(Stmt::Return(None)));
            }
            let expr = self.parse_expr()?;
            self.expect(TokenKind::Semi, "expected ';'")?;
            return Ok(self.arena.alloc_stmt(Stmt::Return(Some(expr))));
        }
        if self.eat(TokenKind::KwBreak) {
            if self.check(TokenKind::Semi) {
                self.expect(TokenKind::Semi, "expected ';'")?;
                return Ok(self.arena.alloc_stmt(Stmt::Break(None)));
            }
            let expr = self.parse_expr()?;
            self.expect(TokenKind::Semi, "expected ';'")?;
            return Ok(self.arena.alloc_stmt(Stmt::Break(Some(expr))));
        }
        if self.eat(TokenKind::KwContinue) {
            self.expect(TokenKind::Semi, "expected ';'")?;
            return Ok(self.arena.alloc_stmt(Stmt::Continue));
        }
        let expr = self.parse_expr()?;
        self.expect(TokenKind::Semi, "expected ';'")?;
        Ok(self.arena.alloc_stmt(Stmt::Expr(expr)))
    }

    fn looks_like_tuple_assign_stmt(&self) -> bool {
        let mut i = self.next_non_layout_idx();
        if self.tokens.get(i).map(|t| t.kind) != Some(TokenKind::LParen) {
            return false;
        }
        let mut depth = 0usize;
        while i < self.tokens.len() {
            let kind = self.tokens[i].kind;
            if !Self::is_layout(kind) {
                match kind {
                    TokenKind::LParen => depth += 1,
                    TokenKind::RParen => {
                        if depth == 0 {
                            return false;
                        }
                        depth -= 1;
                        if depth == 0 {
                            i += 1;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            i += 1;
        }
        while i < self.tokens.len() && Self::is_layout(self.tokens[i].kind) {
            i += 1;
        }
        self.tokens.get(i).map(|t| t.kind) == Some(TokenKind::Assign)
    }

    fn parse_tuple_pattern_items_after_lparen(
        &mut self,
    ) -> Result<Vec<TuplePatternItem>, FrontendError> {
        let mut items = Vec::new();
        loop {
            // M9.4 Wave 2: nested tuple destructuring `(a, (b, c))`.
            let item = if self.eat(TokenKind::LParen) {
                let nested = self.parse_tuple_pattern_items_after_lparen()?;
                TuplePatternItem::Nested(nested)
            } else if self.eat(TokenKind::Underscore) {
                TuplePatternItem::Discard
            } else if self.eat(TokenKind::QuadN) {
                TuplePatternItem::QuadLiteral(QuadVal::N)
            } else if self.eat(TokenKind::QuadF) {
                TuplePatternItem::QuadLiteral(QuadVal::F)
            } else if self.eat(TokenKind::QuadT) {
                TuplePatternItem::QuadLiteral(QuadVal::T)
            } else if self.eat(TokenKind::QuadS) {
                TuplePatternItem::QuadLiteral(QuadVal::S)
            } else if self.eat(TokenKind::KwRef) {
                // M9.5 Wave B: `ref x` — borrow binding in tuple patterns.
                TuplePatternItem::Bind {
                    name: self.expect_symbol()?,
                    capture: CaptureMode::Borrow,
                }
            } else {
                TuplePatternItem::Bind {
                    name: self.expect_symbol()?,
                    capture: CaptureMode::Move,
                }
            };
            if let TuplePatternItem::Bind { name, .. } = item {
                if items.iter().any(|existing| {
                    matches!(existing, TuplePatternItem::Bind { name: existing_name, .. } if *existing_name == name)
                }) {
                    return Err(FrontendError {
                        pos: self.pos(),
                        message: format!(
                            "tuple destructuring pattern cannot repeat '{}'",
                            self.arena.symbol_name(name)
                        ),
                    });
                }
            }
            items.push(item);
            if self.eat(TokenKind::Comma) {
                if self.check(TokenKind::RParen) {
                    break;
                }
                continue;
            }
            break;
        }
        self.expect(
            TokenKind::RParen,
            "expected ')' after tuple destructuring pattern",
        )?;
        if items.len() < 2 {
            return Err(FrontendError {
                pos: self.pos(),
                message: "tuple destructuring pattern requires at least 2 items".to_string(),
            });
        }
        Ok(items)
    }

    fn lower_tuple_pattern_items_to_bind(
        &self,
        items: Vec<TuplePatternItem>,
    ) -> Result<Vec<Option<SymbolId>>, FrontendError> {
        let mut bind_items = Vec::with_capacity(items.len());
        for item in items {
            match item {
                TuplePatternItem::Bind { name, .. } => bind_items.push(Some(name)),
                TuplePatternItem::Discard => bind_items.push(None),
                TuplePatternItem::QuadLiteral(_) => {
                    return Err(FrontendError {
                        pos: self.pos(),
                        message:
                            "quad literal tuple patterns currently require let-else; tuple assignment targets currently support only name/_ items"
                                .to_string(),
                    })
                }
                TuplePatternItem::Nested(_) => {
                    return Err(FrontendError {
                        pos: self.pos(),
                        message:
                            "nested tuple patterns are not supported in tuple assignment targets"
                                .to_string(),
                    })
                }
            }
        }
        Ok(bind_items)
    }

    fn parse_record_pattern_items_after_lbrace(
        &mut self,
    ) -> Result<Vec<RecordPatternItem>, FrontendError> {
        let mut items = Vec::new();
        loop {
            let field = self.expect_symbol()?;
            if items
                .iter()
                .any(|existing: &RecordPatternItem| existing.field == field)
            {
                return Err(FrontendError {
                    pos: self.pos(),
                    message: format!(
                        "record destructuring pattern cannot repeat field '{}'",
                        self.arena.symbol_name(field)
                    ),
                });
            }
            let target = if self.eat(TokenKind::Colon) {
                if self.eat(TokenKind::Underscore) {
                    RecordPatternTarget::Discard
                } else if self.eat(TokenKind::QuadN) {
                    RecordPatternTarget::QuadLiteral(QuadVal::N)
                } else if self.eat(TokenKind::QuadF) {
                    RecordPatternTarget::QuadLiteral(QuadVal::F)
                } else if self.eat(TokenKind::QuadT) {
                    RecordPatternTarget::QuadLiteral(QuadVal::T)
                } else if self.eat(TokenKind::QuadS) {
                    RecordPatternTarget::QuadLiteral(QuadVal::S)
                } else if self.eat(TokenKind::KwRef) {
                    RecordPatternTarget::Bind {
                        name: self.expect_symbol()?,
                        capture: CaptureMode::Borrow,
                    }
                } else {
                    RecordPatternTarget::Bind {
                        name: self.expect_symbol()?,
                        capture: CaptureMode::Move,
                    }
                }
            } else {
                RecordPatternTarget::Bind {
                    name: field,
                    capture: CaptureMode::Move,
                }
            };
            if let RecordPatternTarget::Bind {
                name: target_name, ..
            } = target
            {
                if items.iter().any(|existing| {
                    matches!(
                        existing.target,
                        RecordPatternTarget::Bind { name: existing_name, .. }
                            if existing_name == target_name
                    )
                }) {
                    return Err(FrontendError {
                        pos: self.pos(),
                        message: format!(
                            "record destructuring pattern cannot repeat binding '{}'",
                            self.arena.symbol_name(target_name)
                        ),
                    });
                }
            }
            items.push(RecordPatternItem { field, target });
            if self.eat(TokenKind::Comma) {
                if self.check(TokenKind::RBrace) {
                    break;
                }
                continue;
            }
            break;
        }
        self.expect(
            TokenKind::RBrace,
            "expected '}' after record destructuring pattern",
        )?;
        if items.is_empty() {
            return Err(FrontendError {
                pos: self.pos(),
                message: "record destructuring pattern requires at least 1 field".to_string(),
            });
        }
        Ok(items)
    }

    fn lower_record_pattern_items_to_bind(
        &self,
        items: Vec<RecordPatternItem>,
    ) -> Result<Vec<RecordPatternItem>, FrontendError> {
        for item in &items {
            if matches!(item.target, RecordPatternTarget::QuadLiteral(_)) {
                return Err(FrontendError {
                    pos: self.pos(),
                    message:
                        "quad literal record field patterns currently require let-else; plain record destructuring bind supports only name/_ items"
                            .to_string(),
                });
            }
        }
        Ok(items)
    }

    fn parse_else_return_payload(
        &mut self,
        feature_name: &str,
    ) -> Result<Option<ExprId>, FrontendError> {
        if !self.eat(TokenKind::KwReturn) {
            return Err(FrontendError {
                pos: self.pos(),
                message: format!("{feature_name} currently supports only else return"),
            });
        }
        if self.eat(TokenKind::Semi) {
            return Ok(None);
        }
        let expr = self.parse_expr()?;
        self.expect(TokenKind::Semi, "expected ';'")?;
        Ok(Some(expr))
    }

    fn peek_compound_assign_op(&self) -> Option<BinaryOp> {
        let current = self.next_non_layout_idx();
        let mut next_idx = current + 1;
        while next_idx < self.tokens.len() && Self::is_layout(self.tokens[next_idx].kind) {
            next_idx += 1;
        }
        let next = self.tokens.get(next_idx)?;
        match next.kind {
            TokenKind::PlusAssign => Some(BinaryOp::Add),
            TokenKind::MinusAssign => Some(BinaryOp::Sub),
            TokenKind::StarAssign => Some(BinaryOp::Mul),
            TokenKind::SlashAssign => Some(BinaryOp::Div),
            TokenKind::PercentAssign => Some(BinaryOp::Mod),
            TokenKind::AndAndAssign => Some(BinaryOp::AndAnd),
            TokenKind::OrOrAssign => Some(BinaryOp::OrOr),
            _ => None,
        }
    }

    fn peek_next_non_layout_kind(&self) -> Option<TokenKind> {
        let current = self.next_non_layout_idx();
        let next = self.next_non_layout_idx_from(current + 1);
        self.tokens.get(next).map(|t| t.kind)
    }

    fn parse_if_after_kw_if(&mut self) -> Result<Stmt, FrontendError> {
        let condition = self.parse_expr()?;
        let then_block = self.parse_block()?;
        let else_block = if self.eat(TokenKind::KwElse) {
            if self.eat(TokenKind::KwIf) {
                let nested = self.parse_if_after_kw_if()?;
                vec![self.arena.alloc_stmt(nested)]
            } else {
                self.parse_block()?
            }
        } else {
            Vec::new()
        };
        Ok(Stmt::If {
            condition,
            then_block,
            else_block,
        })
    }

    fn parse_expr(&mut self) -> Result<ExprId, FrontendError> {
        self.parse_where()
    }

    fn parse_where(&mut self) -> Result<ExprId, FrontendError> {
        let tail = self.parse_pipe()?;
        if !self.eat(TokenKind::KwWhere) {
            return Ok(tail);
        }
        let statements = self.parse_where_bindings()?;
        Ok(self
            .arena
            .alloc_expr(Expr::Block(BlockExpr { statements, tail })))
    }

    fn parse_pipe(&mut self) -> Result<ExprId, FrontendError> {
        let mut left = self.parse_impl()?;
        while self.eat(TokenKind::PipeForward) {
            left = self.parse_pipeline_stage(left)?;
        }
        Ok(left)
    }

    fn parse_impl(&mut self) -> Result<ExprId, FrontendError> {
        let mut left = self.parse_or()?;
        while self.eat(TokenKind::Implies) {
            let right = self.parse_or()?;
            left = self
                .arena
                .alloc_expr(Expr::Binary(left, BinaryOp::Implies, right));
        }
        Ok(left)
    }

    fn parse_or(&mut self) -> Result<ExprId, FrontendError> {
        let mut left = self.parse_and()?;
        while self.eat(TokenKind::OrOr) {
            let right = self.parse_and()?;
            left = self
                .arena
                .alloc_expr(Expr::Binary(left, BinaryOp::OrOr, right));
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<ExprId, FrontendError> {
        let mut left = self.parse_eq()?;
        while self.eat(TokenKind::AndAnd) {
            let right = self.parse_eq()?;
            left = self
                .arena
                .alloc_expr(Expr::Binary(left, BinaryOp::AndAnd, right));
        }
        Ok(left)
    }

    fn parse_eq(&mut self) -> Result<ExprId, FrontendError> {
        let mut left = self.parse_cmp()?;
        loop {
            if self.eat(TokenKind::EqEq) {
                let right = self.parse_cmp()?;
                left = self
                    .arena
                    .alloc_expr(Expr::Binary(left, BinaryOp::Eq, right));
                continue;
            }
            if self.eat(TokenKind::KwIs) {
                let right = self.parse_cmp()?;
                left = self
                    .arena
                    .alloc_expr(Expr::Binary(left, BinaryOp::Eq, right));
                continue;
            }
            if self.eat(TokenKind::Ne) {
                let right = self.parse_cmp()?;
                left = self
                    .arena
                    .alloc_expr(Expr::Binary(left, BinaryOp::Ne, right));
                continue;
            }
            break;
        }
        Ok(left)
    }

    fn parse_cmp(&mut self) -> Result<ExprId, FrontendError> {
        let mut left = self.parse_range()?;
        loop {
            let op = if self.eat(TokenKind::LAngle) {
                Some(BinaryOp::Lt)
            } else if self.eat(TokenKind::Le) {
                Some(BinaryOp::Le)
            } else if self.eat(TokenKind::RAngle) {
                Some(BinaryOp::Gt)
            } else if self.eat(TokenKind::Ge) {
                Some(BinaryOp::Ge)
            } else {
                None
            };
            let Some(op) = op else {
                break;
            };
            let right = self.parse_range()?;
            left = self.arena.alloc_expr(Expr::Binary(left, op, right));
        }
        Ok(left)
    }

    fn parse_range(&mut self) -> Result<ExprId, FrontendError> {
        let left = self.parse_add()?;
        if self.eat(TokenKind::DotDotEq) {
            let end = self.parse_add()?;
            return Ok(self.arena.alloc_expr(Expr::Range(RangeExpr {
                start: left,
                end,
                inclusive: true,
            })));
        }
        if self.eat(TokenKind::DotDot) {
            let end = self.parse_add()?;
            return Ok(self.arena.alloc_expr(Expr::Range(RangeExpr {
                start: left,
                end,
                inclusive: false,
            })));
        }
        Ok(left)
    }

    fn parse_add(&mut self) -> Result<ExprId, FrontendError> {
        let mut left = self.parse_mul()?;
        loop {
            if self.eat(TokenKind::Plus) {
                let right = self.parse_mul()?;
                left = self
                    .arena
                    .alloc_expr(Expr::Binary(left, BinaryOp::Add, right));
                continue;
            }
            if self.eat(TokenKind::Minus) {
                let right = self.parse_mul()?;
                left = self
                    .arena
                    .alloc_expr(Expr::Binary(left, BinaryOp::Sub, right));
                continue;
            }
            break;
        }
        Ok(left)
    }

    fn parse_mul(&mut self) -> Result<ExprId, FrontendError> {
        let mut left = self.parse_unary()?;
        loop {
            if self.eat(TokenKind::Star) {
                let right = self.parse_unary()?;
                left = self
                    .arena
                    .alloc_expr(Expr::Binary(left, BinaryOp::Mul, right));
                continue;
            }
            if self.eat(TokenKind::Slash) {
                let right = self.parse_unary()?;
                left = self
                    .arena
                    .alloc_expr(Expr::Binary(left, BinaryOp::Div, right));
                continue;
            }
            if self.eat(TokenKind::Percent) {
                let right = self.parse_unary()?;
                left = self
                    .arena
                    .alloc_expr(Expr::Binary(left, BinaryOp::Mod, right));
                continue;
            }
            break;
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<ExprId, FrontendError> {
        if self.eat(TokenKind::Bang) {
            let inner = self.parse_unary()?;
            return Ok(self.arena.alloc_expr(Expr::Unary(UnaryOp::Not, inner)));
        }
        if self.eat(TokenKind::Plus) {
            let inner = self.parse_unary()?;
            return Ok(self.arena.alloc_expr(Expr::Unary(UnaryOp::Pos, inner)));
        }
        if self.eat(TokenKind::Minus) {
            let inner = self.parse_unary()?;
            return Ok(self.arena.alloc_expr(Expr::Unary(UnaryOp::Neg, inner)));
        }
        self.parse_primary()
    }

    fn parse_pipeline_stage(&mut self, input: ExprId) -> Result<ExprId, FrontendError> {
        if self.eat(TokenKind::LParen) {
            return self.parse_short_lambda_apply_after_lparen(Some(input), true);
        }
        if !self.check(TokenKind::Ident) {
            return Err(FrontendError {
                pos: self.pos(),
                message: "pipeline stage must start with function name or call".to_string(),
            });
        }

        let name = self.expect_symbol()?;
        let mut args = vec![CallArg {
            name: None,
            value: input,
        }];
        if self.eat(TokenKind::LParen) {
            args.extend(self.parse_call_args()?);
            self.expect(TokenKind::RParen, "expected ')'")?;
        }
        Ok(self.arena.alloc_expr(Expr::Call(name, args)))
    }

    fn parse_primary(&mut self) -> Result<ExprId, FrontendError> {
        let mut expr = self.parse_primary_atom()?;
        loop {
            if self.eat(TokenKind::Dot) {
                let field = self.expect_symbol()?;
                if !self.eat(TokenKind::LParen) {
                    expr = self
                        .arena
                        .alloc_expr(Expr::RecordField(RecordFieldExpr { base: expr, field }));
                    continue;
                }
                let mut args = vec![CallArg {
                    name: None,
                    value: expr,
                }];
                args.extend(self.parse_call_args()?);
                self.expect(TokenKind::RParen, "expected ')' after UFCS method call")?;
                expr = self.arena.alloc_expr(Expr::Call(field, args));
                continue;
            }
            if self.eat(TokenKind::LBracket) {
                let index = self.parse_expr()?;
                self.expect(TokenKind::RBracket, "expected ']' after sequence index")?;
                expr = self
                    .arena
                    .alloc_expr(Expr::SequenceIndex(SequenceIndexExpr { base: expr, index }));
                continue;
            }
            if self.eat(TokenKind::KwWith) {
                self.expect(
                    TokenKind::LBrace,
                    "expected '{' after 'with' in record copy-with",
                )?;
                let fields = self.parse_record_init_fields_after_lbrace()?;
                expr = self
                    .arena
                    .alloc_expr(Expr::RecordUpdate(RecordUpdateExpr { base: expr, fields }));
                continue;
            }
            break;
        }
        Ok(expr)
    }

    fn parse_primary_atom(&mut self) -> Result<ExprId, FrontendError> {
        if self.eat(TokenKind::KwIf) || self.eat(TokenKind::KwWhen) {
            return self.parse_if_expr_after_kw_if();
        }
        if self.eat(TokenKind::KwMatch) {
            return self.parse_match_expr_after_kw_match();
        }
        if self.eat(TokenKind::KwLoop) {
            return self.parse_loop_expr_after_kw_loop();
        }
        if self.check(TokenKind::LBrace) {
            return self.parse_block_expr();
        }
        if self.eat(TokenKind::LParen) {
            if self.starts_short_lambda_head() {
                return self.parse_short_lambda_apply_after_lparen(None, false);
            }
            return self.parse_paren_expr_or_tuple();
        }
        if self.eat(TokenKind::LBracket) {
            return self.parse_bracket_expr();
        }
        if self.eat(TokenKind::QuadN) {
            return Ok(self.arena.alloc_expr(Expr::QuadLiteral(QuadVal::N)));
        }
        if self.eat(TokenKind::QuadF) {
            return Ok(self.arena.alloc_expr(Expr::QuadLiteral(QuadVal::F)));
        }
        if self.eat(TokenKind::QuadT) {
            return Ok(self.arena.alloc_expr(Expr::QuadLiteral(QuadVal::T)));
        }
        if self.eat(TokenKind::QuadS) {
            return Ok(self.arena.alloc_expr(Expr::QuadLiteral(QuadVal::S)));
        }
        if self.eat(TokenKind::KwTrue) {
            return Ok(self.arena.alloc_expr(Expr::BoolLiteral(true)));
        }
        if self.eat(TokenKind::KwFalse) {
            return Ok(self.arena.alloc_expr(Expr::BoolLiteral(false)));
        }
        if self.check(TokenKind::String) {
            let spelling = self.advance().text;
            return Ok(self.arena.alloc_expr(Expr::TextLiteral(TextLiteral {
                family: TextLiteralFamily::DoubleQuotedUtf8,
                spelling,
            })));
        }
        if self.check(TokenKind::Num) {
            let text = self.advance().text;
            return self.parse_numeric_literal_expr(&text);
        }
        if self.check(TokenKind::Ident) {
            let name = self.expect_symbol()?;
            if self.eat(TokenKind::PathSep) {
                let variant_name = self.expect_symbol()?;
                let payload = if self.eat(TokenKind::LParen) {
                    self.parse_adt_ctor_payload_exprs()?
                } else {
                    Vec::new()
                };
                return Ok(self.arena.alloc_expr(Expr::AdtCtor(AdtCtorExpr {
                    adt_name: name,
                    variant_name,
                    payload,
                })));
            }
            if self.eat(TokenKind::LParen) {
                let args = self.parse_call_args()?;
                self.expect(TokenKind::RParen, "expected ')'")?;
                return Ok(self.arena.alloc_expr(Expr::Call(name, args)));
            }
            if self.starts_record_literal_head() {
                self.expect(
                    TokenKind::LBrace,
                    "expected '{' after record literal type name",
                )?;
                return self.parse_record_literal_after_name(name);
            }
            return Ok(self.arena.alloc_expr(Expr::Var(name)));
        }
        Err(FrontendError {
            pos: self.pos(),
            message: "expected primary expression".to_string(),
        })
    }

    fn parse_bracket_expr(&mut self) -> Result<ExprId, FrontendError> {
        let mut items = Vec::new();
        while !self.check(TokenKind::RBracket) {
            items.push(self.parse_expr()?);
            if self.eat(TokenKind::Comma) {
                if self.check(TokenKind::RBracket) {
                    break;
                }
                continue;
            }
            break;
        }
        self.expect(TokenKind::RBracket, "expected ']' after sequence literal")?;
        Ok(self
            .arena
            .alloc_expr(Expr::SequenceLiteral(SequenceLiteral {
                family: SequenceCollectionFamily::OrderedSequence,
                items,
            })))
    }

    fn parse_adt_ctor_payload_exprs(&mut self) -> Result<Vec<ExprId>, FrontendError> {
        let mut payload = Vec::new();
        if self.check(TokenKind::RParen) {
            return Err(FrontendError {
                pos: self.pos(),
                message: "enum constructor payload cannot be empty parentheses; omit '()' for unit variant".to_string(),
            });
        }
        loop {
            payload.push(self.parse_expr()?);
            if self.eat(TokenKind::Comma) {
                if self.check(TokenKind::RParen) {
                    break;
                }
                continue;
            }
            break;
        }
        self.expect(
            TokenKind::RParen,
            "expected ')' after enum constructor payload",
        )?;
        Ok(payload)
    }

    fn parse_paren_expr_or_tuple(&mut self) -> Result<ExprId, FrontendError> {
        if self.check(TokenKind::RParen) {
            return Err(FrontendError {
                pos: self.pos(),
                message: "empty tuple literal is not supported in v0".to_string(),
            });
        }
        let first = self.parse_expr()?;
        if !self.eat(TokenKind::Comma) {
            self.expect(TokenKind::RParen, "expected ')'")?;
            return Ok(first);
        }

        let mut items = vec![first, self.parse_expr()?];
        while self.eat(TokenKind::Comma) {
            if self.check(TokenKind::RParen) {
                break;
            }
            items.push(self.parse_expr()?);
        }
        self.expect(TokenKind::RParen, "expected ')' after tuple literal")?;
        Ok(self.arena.alloc_expr(Expr::Tuple(items)))
    }

    fn parse_record_literal_after_name(&mut self, name: SymbolId) -> Result<ExprId, FrontendError> {
        let fields = self.parse_record_init_fields_after_lbrace()?;
        Ok(self
            .arena
            .alloc_expr(Expr::RecordLiteral(RecordLiteralExpr { name, fields })))
    }

    fn parse_record_init_fields_after_lbrace(
        &mut self,
    ) -> Result<Vec<RecordInitField>, FrontendError> {
        let mut fields = Vec::new();
        while !self.check(TokenKind::RBrace) {
            let field_name = self.expect_symbol()?;
            let value = if self.eat(TokenKind::Colon) {
                self.parse_expr()?
            } else {
                self.arena.alloc_expr(Expr::Var(field_name))
            };
            fields.push(RecordInitField {
                name: field_name,
                value,
            });
            if self.eat(TokenKind::Comma) {
                if self.check(TokenKind::RBrace) {
                    break;
                }
                continue;
            }
            break;
        }
        self.expect(TokenKind::RBrace, "expected '}' after record field list")?;
        Ok(fields)
    }

    fn starts_record_literal_head(&self) -> bool {
        let lbrace_idx = self.next_non_layout_idx();
        if lbrace_idx >= self.tokens.len() || self.tokens[lbrace_idx].kind != TokenKind::LBrace {
            return false;
        }
        let field_idx = self.next_non_layout_idx_from(lbrace_idx + 1);
        if field_idx >= self.tokens.len() {
            return false;
        }
        if self.tokens[field_idx].kind == TokenKind::RBrace {
            return true;
        }
        if self.tokens[field_idx].kind != TokenKind::Ident {
            return false;
        }
        let next_idx = self.next_non_layout_idx_from(field_idx + 1);
        next_idx < self.tokens.len()
            && matches!(
                self.tokens[next_idx].kind,
                TokenKind::Colon | TokenKind::Comma | TokenKind::RBrace
            )
    }

    fn starts_short_lambda_head(&self) -> bool {
        self.check(TokenKind::Ident) && self.peek_next_kind() == Some(TokenKind::FatArrow)
    }

    fn parse_call_args(&mut self) -> Result<Vec<CallArg>, FrontendError> {
        let mut args = Vec::new();
        let mut named_seen = false;
        if self.check(TokenKind::RParen) {
            return Ok(args);
        }
        loop {
            let arg = if self.check(TokenKind::Ident)
                && self.peek_next_kind() == Some(TokenKind::Assign)
            {
                named_seen = true;
                let name = self.expect_symbol()?;
                self.expect(TokenKind::Assign, "expected '=' in named argument")?;
                let value = self.parse_expr()?;
                CallArg {
                    name: Some(name),
                    value,
                }
            } else {
                if named_seen {
                    return Err(FrontendError {
                        pos: self.pos(),
                        message: "positional arguments cannot follow named arguments".to_string(),
                    });
                }
                CallArg {
                    name: None,
                    value: self.parse_expr()?,
                }
            };
            args.push(arg);
            if self.eat(TokenKind::Comma) {
                continue;
            }
            break;
        }
        Ok(args)
    }

    fn parse_where_bindings(&mut self) -> Result<Vec<StmtId>, FrontendError> {
        let mut statements = Vec::new();
        loop {
            let name = self.expect_symbol()?;
            let ty = if self.eat(TokenKind::Colon) {
                Some(self.parse_type()?)
            } else {
                None
            };
            self.expect(TokenKind::Assign, "expected '=' in where binding")?;
            let value = self.parse_expr()?;
            statements.push(self.arena.alloc_stmt(Stmt::Let {
                name,
                is_mut: false,
                ty,
                value,
            }));
            if self.eat(TokenKind::Comma) {
                continue;
            }
            break;
        }
        Ok(statements)
    }

    fn parse_short_lambda_apply_after_lparen(
        &mut self,
        pipeline_input: Option<ExprId>,
        from_pipeline: bool,
    ) -> Result<ExprId, FrontendError> {
        if !self.starts_short_lambda_head() {
            return Err(FrontendError {
                pos: self.pos(),
                message: if from_pipeline {
                    "pipeline short lambda must use form '(x => expr)'".to_string()
                } else {
                    "expected parenthesized expression or short lambda".to_string()
                },
            });
        }

        let param = self.expect_symbol()?;
        self.expect(
            TokenKind::FatArrow,
            "expected '=>' after short lambda parameter",
        )?;
        let body = self.parse_expr()?;
        self.expect(TokenKind::RParen, "expected ')' after short lambda body")?;

        if from_pipeline {
            self.ensure_short_lambda_capture_free(body, param)?;
            let arg =
                pipeline_input.expect("pipeline input must be provided for pipeline short lambda");
            return self.build_short_lambda_apply(param, body, arg);
        }

        if self.check(TokenKind::LParen) {
            self.ensure_short_lambda_capture_free(body, param)?;
            let arg = self.parse_short_lambda_immediate_arg()?;
            return self.build_short_lambda_apply(param, body, arg);
        }

        self.build_first_class_closure_literal(param, body)
    }

    fn parse_short_lambda_immediate_arg(&mut self) -> Result<ExprId, FrontendError> {
        self.expect(
            TokenKind::LParen,
            "short lambda direct call currently requires form '(x => expr)(arg)'",
        )?;
        if self.check(TokenKind::RParen) {
            return Err(FrontendError {
                pos: self.pos(),
                message: "short lambda immediate call requires exactly one argument".to_string(),
            });
        }
        let arg = self.parse_expr()?;
        if self.eat(TokenKind::Comma) {
            return Err(FrontendError {
                pos: self.pos(),
                message: "short lambda v0 currently supports exactly one argument".to_string(),
            });
        }
        self.expect(
            TokenKind::RParen,
            "expected ')' after short lambda argument",
        )?;
        Ok(arg)
    }

    fn build_first_class_closure_literal(
        &mut self,
        param: SymbolId,
        body: ExprId,
    ) -> Result<ExprId, FrontendError> {
        let captures = self.collect_short_lambda_captures(body, param)?;
        Ok(self.arena.alloc_expr(Expr::Closure(ClosureLiteral {
            family: ClosureValueFamily::UnaryDirect,
            capture: ClosureCapturePolicy::Immutable,
            param,
            param_ty: None,
            ret_ty: None,
            captures,
            body,
        })))
    }

    fn build_short_lambda_apply(
        &mut self,
        param: SymbolId,
        body: ExprId,
        arg: ExprId,
    ) -> Result<ExprId, FrontendError> {
        let binding = self.arena.alloc_stmt(Stmt::Let {
            name: param,
            is_mut: false,
            ty: None,
            value: arg,
        });
        Ok(self.arena.alloc_expr(Expr::Block(BlockExpr {
            statements: vec![binding],
            tail: body,
        })))
    }

    fn collect_short_lambda_captures(
        &self,
        body: ExprId,
        param: SymbolId,
    ) -> Result<Vec<SymbolId>, FrontendError> {
        let mut scopes = vec![vec![param]];
        let mut captures = Vec::new();
        self.collect_short_lambda_expr_captures(body, &mut scopes, &mut captures)?;
        Ok(captures)
    }

    fn collect_short_lambda_expr_captures(
        &self,
        expr_id: ExprId,
        scopes: &mut Vec<Vec<SymbolId>>,
        captures: &mut Vec<SymbolId>,
    ) -> Result<(), FrontendError> {
        match self.arena.expr(expr_id) {
            Expr::QuadLiteral(_)
            | Expr::BoolLiteral(_)
            | Expr::TextLiteral(_)
            | Expr::NumericLiteral(_) => Ok(()),
            Expr::Range(range_expr) => {
                self.collect_short_lambda_expr_captures(range_expr.start, scopes, captures)?;
                self.collect_short_lambda_expr_captures(range_expr.end, scopes, captures)
            }
            Expr::Tuple(items) => {
                for item in items {
                    self.collect_short_lambda_expr_captures(*item, scopes, captures)?;
                }
                Ok(())
            }
            Expr::SequenceLiteral(sequence) => {
                for item in &sequence.items {
                    self.collect_short_lambda_expr_captures(*item, scopes, captures)?;
                }
                Ok(())
            }
            Expr::RecordLiteral(record) => {
                for field in &record.fields {
                    self.collect_short_lambda_expr_captures(field.value, scopes, captures)?;
                }
                Ok(())
            }
            Expr::AdtCtor(ctor) => {
                for item in &ctor.payload {
                    self.collect_short_lambda_expr_captures(*item, scopes, captures)?;
                }
                Ok(())
            }
            Expr::RecordField(field_expr) => {
                self.collect_short_lambda_expr_captures(field_expr.base, scopes, captures)
            }
            Expr::SequenceIndex(index_expr) => {
                self.collect_short_lambda_expr_captures(index_expr.base, scopes, captures)?;
                self.collect_short_lambda_expr_captures(index_expr.index, scopes, captures)
            }
            Expr::Closure(_) => Err(FrontendError {
                pos: self.pos(),
                message:
                    "nested first-class closure literals are not yet admitted before M8.4 Wave 2"
                        .to_string(),
            }),
            Expr::RecordUpdate(update_expr) => {
                self.collect_short_lambda_expr_captures(update_expr.base, scopes, captures)?;
                for field in &update_expr.fields {
                    self.collect_short_lambda_expr_captures(field.value, scopes, captures)?;
                }
                Ok(())
            }
            Expr::Var(name) => {
                if scopes.iter().rev().any(|scope| scope.contains(name)) {
                    Ok(())
                } else {
                    if !captures.contains(name) {
                        captures.push(*name);
                    }
                    Ok(())
                }
            }
            Expr::Call(_, args) => {
                for arg in args {
                    self.collect_short_lambda_expr_captures(arg.value, scopes, captures)?;
                }
                Ok(())
            }
            Expr::Unary(_, inner) => self.collect_short_lambda_expr_captures(*inner, scopes, captures),
            Expr::Binary(lhs, _, rhs) => {
                self.collect_short_lambda_expr_captures(*lhs, scopes, captures)?;
                self.collect_short_lambda_expr_captures(*rhs, scopes, captures)
            }
            Expr::Block(block) => self.collect_short_lambda_block_captures(block, scopes, captures),
            Expr::If(if_expr) => {
                self.collect_short_lambda_expr_captures(if_expr.condition, scopes, captures)?;
                self.collect_short_lambda_block_captures(&if_expr.then_block, scopes, captures)?;
                self.collect_short_lambda_block_captures(&if_expr.else_block, scopes, captures)
            }
            Expr::Match(match_expr) => {
                self.collect_short_lambda_expr_captures(match_expr.scrutinee, scopes, captures)?;
                for arm in &match_expr.arms {
                    scopes.push(self.short_lambda_match_pattern_bindings(&arm.pat));
                    if let Some(guard) = arm.guard {
                        self.collect_short_lambda_expr_captures(guard, scopes, captures)?;
                    }
                    self.collect_short_lambda_block_captures(&arm.block, scopes, captures)?;
                    let _ = scopes.pop();
                }
                if let Some(default) = &match_expr.default {
                    self.collect_short_lambda_block_captures(default, scopes, captures)?;
                }
                Ok(())
            }
            Expr::IfLet(_) => Err(FrontendError {
                pos: self.pos(),
                message:
                    "first-class closure literals do not yet admit if-let expressions in the closure body"
                        .to_string(),
            }),
            Expr::Loop(_) => Err(FrontendError {
                pos: self.pos(),
                message:
                    "first-class closure literals do not yet admit loop expressions in the closure body"
                        .to_string(),
            }),
        }
    }

    fn collect_short_lambda_block_captures(
        &self,
        block: &BlockExpr,
        scopes: &mut Vec<Vec<SymbolId>>,
        captures: &mut Vec<SymbolId>,
    ) -> Result<(), FrontendError> {
        scopes.push(Vec::new());
        for stmt_id in &block.statements {
            self.collect_short_lambda_stmt_captures(*stmt_id, scopes, captures)?;
        }
        self.collect_short_lambda_expr_captures(block.tail, scopes, captures)?;
        let _ = scopes.pop();
        Ok(())
    }

    fn collect_short_lambda_stmt_captures(
        &self,
        stmt_id: StmtId,
        scopes: &mut Vec<Vec<SymbolId>>,
        captures: &mut Vec<SymbolId>,
    ) -> Result<(), FrontendError> {
        match self.arena.stmt(stmt_id) {
            Stmt::Const { name, value, .. } => {
                self.collect_short_lambda_expr_captures(*value, scopes, captures)?;
                if let Some(scope) = scopes.last_mut() {
                    scope.push(*name);
                }
                Ok(())
            }
            Stmt::Let { name, value, .. } => {
                self.collect_short_lambda_expr_captures(*value, scopes, captures)?;
                if let Some(scope) = scopes.last_mut() {
                    scope.push(*name);
                }
                Ok(())
            }
            Stmt::LetTuple { items, value, .. } => {
                self.collect_short_lambda_expr_captures(*value, scopes, captures)?;
                if let Some(scope) = scopes.last_mut() {
                    scope.extend(items.iter().filter_map(|item| match item {
                        TuplePatternItem::Bind { name, .. } => Some(*name),
                        _ => None,
                    }));
                }
                Ok(())
            }
            Stmt::Discard { value, .. } => {
                self.collect_short_lambda_expr_captures(*value, scopes, captures)
            }
            Stmt::Expr(expr_id) => self.collect_short_lambda_expr_captures(*expr_id, scopes, captures),
            _ => Err(FrontendError {
                pos: self.pos(),
                message:
                    "first-class closure literals currently support only expression-compatible block forms"
                        .to_string(),
            }),
        }
    }

    fn ensure_short_lambda_capture_free(
        &self,
        body: ExprId,
        param: SymbolId,
    ) -> Result<(), FrontendError> {
        let mut scopes = vec![vec![param]];
        self.ensure_short_lambda_expr_capture_free(body, &mut scopes)
    }

    fn ensure_short_lambda_expr_capture_free(
        &self,
        expr_id: ExprId,
        scopes: &mut Vec<Vec<SymbolId>>,
    ) -> Result<(), FrontendError> {
        match self.arena.expr(expr_id) {
            Expr::QuadLiteral(_)
            | Expr::BoolLiteral(_)
            | Expr::TextLiteral(_)
            | Expr::NumericLiteral(_) => Ok(()),
            Expr::Range(range_expr) => {
                self.ensure_short_lambda_expr_capture_free(range_expr.start, scopes)?;
                self.ensure_short_lambda_expr_capture_free(range_expr.end, scopes)
            }
            Expr::Tuple(items) => {
                for item in items {
                    self.ensure_short_lambda_expr_capture_free(*item, scopes)?;
                }
                Ok(())
            }
            Expr::SequenceLiteral(sequence) => {
                for item in &sequence.items {
                    self.ensure_short_lambda_expr_capture_free(*item, scopes)?;
                }
                Ok(())
            }
            Expr::RecordLiteral(record) => {
                for field in &record.fields {
                    self.ensure_short_lambda_expr_capture_free(field.value, scopes)?;
                }
                Ok(())
            }
            Expr::AdtCtor(ctor) => {
                for item in &ctor.payload {
                    self.ensure_short_lambda_expr_capture_free(*item, scopes)?;
                }
                Ok(())
            }
            Expr::RecordField(field_expr) => {
                self.ensure_short_lambda_expr_capture_free(field_expr.base, scopes)
            }
            Expr::SequenceIndex(index_expr) => {
                self.ensure_short_lambda_expr_capture_free(index_expr.base, scopes)?;
                self.ensure_short_lambda_expr_capture_free(index_expr.index, scopes)
            }
            Expr::Closure(_) => Err(FrontendError {
                pos: self.pos(),
                message:
                    "short lambda v0 does not currently allow nested first-class closure values in the lambda body"
                        .to_string(),
            }),
            Expr::RecordUpdate(update_expr) => {
                self.ensure_short_lambda_expr_capture_free(update_expr.base, scopes)?;
                for field in &update_expr.fields {
                    self.ensure_short_lambda_expr_capture_free(field.value, scopes)?;
                }
                Ok(())
            }
            Expr::Var(name) => {
                if scopes.iter().rev().any(|scope| scope.contains(name)) {
                    Ok(())
                } else {
                    Err(FrontendError {
                        pos: self.pos(),
                        message: format!(
                            "short lambda v0 is capture-free only; body may not reference non-local '{}'",
                            self.arena.symbol_name(*name)
                        ),
                    })
                }
            }
            Expr::Call(_, args) => {
                for arg in args {
                    self.ensure_short_lambda_expr_capture_free(arg.value, scopes)?;
                }
                Ok(())
            }
            Expr::Unary(_, inner) => self.ensure_short_lambda_expr_capture_free(*inner, scopes),
            Expr::Binary(lhs, _, rhs) => {
                self.ensure_short_lambda_expr_capture_free(*lhs, scopes)?;
                self.ensure_short_lambda_expr_capture_free(*rhs, scopes)
            }
            Expr::Block(block) => self.ensure_short_lambda_block_capture_free(block, scopes),
            Expr::If(if_expr) => {
                self.ensure_short_lambda_expr_capture_free(if_expr.condition, scopes)?;
                self.ensure_short_lambda_block_capture_free(&if_expr.then_block, scopes)?;
                self.ensure_short_lambda_block_capture_free(&if_expr.else_block, scopes)
            }
            Expr::Match(match_expr) => {
                self.ensure_short_lambda_expr_capture_free(match_expr.scrutinee, scopes)?;
                for arm in &match_expr.arms {
                    scopes.push(self.short_lambda_match_pattern_bindings(&arm.pat));
                    if let Some(guard) = arm.guard {
                        self.ensure_short_lambda_expr_capture_free(guard, scopes)?;
                    }
                    self.ensure_short_lambda_block_capture_free(&arm.block, scopes)?;
                    let _ = scopes.pop();
                }
                if let Some(default) = &match_expr.default {
                    self.ensure_short_lambda_block_capture_free(default, scopes)?;
                }
                Ok(())
            }
            Expr::IfLet(_) => Err(FrontendError {
                pos: self.pos(),
                message:
                    "short lambda v0 does not currently allow if-let expressions in the lambda body"
                        .to_string(),
            }),
            Expr::Loop(_) => Err(FrontendError {
                pos: self.pos(),
                message:
                    "short lambda v0 does not currently allow loop expressions in the lambda body"
                        .to_string(),
            }),
        }
    }

    fn short_lambda_match_pattern_bindings(&self, pat: &MatchPattern) -> Vec<SymbolId> {
        match pat {
            MatchPattern::Quad(_) | MatchPattern::Wildcard | MatchPattern::IntRange(_) => {
                Vec::new()
            }
            MatchPattern::Adt(adt_pat) => adt_pat
                .items
                .iter()
                .filter_map(|item| match item {
                    AdtPatternItem::Bind { name, .. } => Some(*name),
                    AdtPatternItem::Discard => None,
                })
                .collect(),
            MatchPattern::Or(alts) => {
                // Bindings from the first alternative (Wave 2/3 will enforce same names across alts).
                alts.first()
                    .map(|p| self.short_lambda_match_pattern_bindings(p))
                    .unwrap_or_default()
            }
        }
    }

    fn ensure_short_lambda_block_capture_free(
        &self,
        block: &BlockExpr,
        scopes: &mut Vec<Vec<SymbolId>>,
    ) -> Result<(), FrontendError> {
        scopes.push(Vec::new());
        for stmt_id in &block.statements {
            self.ensure_short_lambda_stmt_capture_free(*stmt_id, scopes)?;
        }
        self.ensure_short_lambda_expr_capture_free(block.tail, scopes)?;
        let _ = scopes.pop();
        Ok(())
    }

    fn ensure_short_lambda_stmt_capture_free(
        &self,
        stmt_id: StmtId,
        scopes: &mut Vec<Vec<SymbolId>>,
    ) -> Result<(), FrontendError> {
        match self.arena.stmt(stmt_id) {
            Stmt::Const { name, value, .. } => {
                self.ensure_short_lambda_expr_capture_free(*value, scopes)?;
                if let Some(scope) = scopes.last_mut() {
                    scope.push(*name);
                }
                Ok(())
            }
            Stmt::Let { name, value, .. } => {
                self.ensure_short_lambda_expr_capture_free(*value, scopes)?;
                if let Some(scope) = scopes.last_mut() {
                    scope.push(*name);
                }
                Ok(())
            }
            Stmt::LetTuple { items, value, .. } => {
                self.ensure_short_lambda_expr_capture_free(*value, scopes)?;
                if let Some(scope) = scopes.last_mut() {
                    scope.extend(items.iter().filter_map(|item| match item {
                        TuplePatternItem::Bind { name, .. } => Some(*name),
                        _ => None,
                    }));
                }
                Ok(())
            }
            Stmt::Discard { value, .. } => {
                self.ensure_short_lambda_expr_capture_free(*value, scopes)
            }
            Stmt::Expr(expr_id) => self.ensure_short_lambda_expr_capture_free(*expr_id, scopes),
            _ => Err(FrontendError {
                pos: self.pos(),
                message:
                    "short lambda body currently supports only expression-compatible block forms"
                        .to_string(),
            }),
        }
    }

    fn peek_next_kind(&self) -> Option<TokenKind> {
        let mut i = self.next_non_layout_idx();
        i += 1;
        while i < self.tokens.len() && Self::is_layout(self.tokens[i].kind) {
            i += 1;
        }
        self.tokens.get(i).map(|t| t.kind)
    }

    fn parse_block_expr(&mut self) -> Result<ExprId, FrontendError> {
        let block = self.parse_value_block()?;
        Ok(self.arena.alloc_expr(Expr::Block(block)))
    }

    fn starts_stmt_only_in_block_expr(&self) -> bool {
        self.check(TokenKind::KwLet) || self.check(TokenKind::KwConst)
    }

    fn parse_if_expr_after_kw_if(&mut self) -> Result<ExprId, FrontendError> {
        // M9.4 Wave 2: if-let expression `if let Pattern = expr { ... } else { ... }`
        if self.eat(TokenKind::KwLet) {
            let pattern = self.parse_match_pattern()?;
            self.expect(TokenKind::Assign, "expected '=' after pattern in if-let")?;
            let value = self.parse_expr()?;
            let then_block = self.parse_value_block()?;
            if !self.eat(TokenKind::KwElse) {
                return Err(FrontendError {
                    pos: self.pos(),
                    message: "if-let expression requires explicit else branch".to_string(),
                });
            }
            let else_block = self.parse_value_block()?;
            return Ok(self.arena.alloc_expr(Expr::IfLet(IfLetExpr {
                pattern,
                value,
                then_block,
                else_block,
            })));
        }
        let condition = self.parse_expr()?;
        let then_block = self.parse_value_block()?;
        if !self.eat(TokenKind::KwElse) {
            return Err(FrontendError {
                pos: self.pos(),
                message: "if expression requires explicit else branch".to_string(),
            });
        }
        if self.check(TokenKind::KwIf) {
            self.expect(TokenKind::KwIf, "expected 'if' after 'else'")?;
            let nested_if = self.parse_if_expr_after_kw_if()?;
            return Ok(self.arena.alloc_expr(Expr::If(IfExpr {
                condition,
                then_block,
                else_block: BlockExpr {
                    statements: Vec::new(),
                    tail: nested_if,
                },
            })));
        }
        let else_block = self.parse_value_block()?;
        Ok(self.arena.alloc_expr(Expr::If(IfExpr {
            condition,
            then_block,
            else_block,
        })))
    }

    fn parse_match_stmt_after_kw_match(&mut self) -> Result<StmtId, FrontendError> {
        let scrutinee = self.parse_expr()?;
        self.expect(TokenKind::LBrace, "expected '{' after match expr")?;
        let mut arms = Vec::new();
        let mut default: Option<Vec<StmtId>> = None;
        while !self.check(TokenKind::RBrace) {
            if self.eat(TokenKind::Underscore) {
                if self.check(TokenKind::KwIf) {
                    return Err(FrontendError {
                        pos: self.pos(),
                        message: "default '_' arm in match currently cannot have guard".to_string(),
                    });
                }
                self.expect(TokenKind::FatArrow, "expected '=>' after '_'")?;
                let block = self.parse_block()?;
                default = Some(block);
                continue;
            }
            let pat = self.parse_match_pattern()?;
            let guard = self.parse_match_guard_opt()?;
            self.expect(TokenKind::FatArrow, "expected '=>' after match pattern")?;
            let block = self.parse_block()?;
            arms.push(MatchArm { pat, guard, block });
        }
        self.expect(TokenKind::RBrace, "expected '}' after match arms")?;
        Ok(self.arena.alloc_stmt(Stmt::Match {
            scrutinee,
            arms,
            default: default.unwrap_or_default(),
        }))
    }

    fn parse_match_expr_after_kw_match(&mut self) -> Result<ExprId, FrontendError> {
        let scrutinee = self.parse_expr()?;
        self.expect(TokenKind::LBrace, "expected '{' after match expr")?;
        let mut arms = Vec::new();
        let mut default: Option<BlockExpr> = None;
        while !self.check(TokenKind::RBrace) {
            if self.eat(TokenKind::Underscore) {
                if self.check(TokenKind::KwIf) {
                    return Err(FrontendError {
                        pos: self.pos(),
                        message: "default '_' arm in match currently cannot have guard".to_string(),
                    });
                }
                self.expect(TokenKind::FatArrow, "expected '=>' after '_'")?;
                let block = self.parse_value_block()?;
                default = Some(block);
                continue;
            }
            let pat = self.parse_match_pattern()?;
            let guard = self.parse_match_guard_opt()?;
            self.expect(TokenKind::FatArrow, "expected '=>' after match pattern")?;
            let block = self.parse_value_block()?;
            arms.push(MatchExprArm { pat, guard, block });
        }
        self.expect(TokenKind::RBrace, "expected '}' after match arms")?;
        Ok(self.arena.alloc_expr(Expr::Match(MatchExpr {
            scrutinee,
            arms,
            default,
        })))
    }

    fn parse_match_pattern(&mut self) -> Result<MatchPattern, FrontendError> {
        let first = self.parse_match_pattern_single()?;
        // M9.4 Wave 2: or-pattern — collect alternatives separated by `|`.
        if !self.check(TokenKind::Pipe) {
            return Ok(first);
        }
        let mut alts = vec![first];
        while self.eat(TokenKind::Pipe) {
            alts.push(self.parse_match_pattern_single()?);
        }
        Ok(MatchPattern::Or(alts))
    }

    /// Parse a single (non-or) match pattern.
    fn parse_match_pattern_single(&mut self) -> Result<MatchPattern, FrontendError> {
        // M9.4 Wave 2: wildcard `_`
        if self.eat(TokenKind::Underscore) {
            return Ok(MatchPattern::Wildcard);
        }
        if self.eat(TokenKind::QuadN) {
            return Ok(MatchPattern::Quad(QuadVal::N));
        } else if self.eat(TokenKind::QuadF) {
            return Ok(MatchPattern::Quad(QuadVal::F));
        } else if self.eat(TokenKind::QuadT) {
            return Ok(MatchPattern::Quad(QuadVal::T));
        } else if self.eat(TokenKind::QuadS) {
            return Ok(MatchPattern::Quad(QuadVal::S));
        }
        // M9.4 Wave 2: integer range patterns `1..=5` or `1..5`
        if self.check(TokenKind::Num) {
            let text = self.peek().text.clone();
            // Only admit plain integer literals (no suffix, no decimal point).
            let is_plain_int =
                !text.contains('.') && !text.contains("i32") && !text.contains("u32");
            if is_plain_int {
                // Lookahead: is the token after the number `..` or `..=`?
                // We need to consume the number then check.
                let num_text = self.advance().text;
                if self.check(TokenKind::DotDot) || self.check(TokenKind::DotDotEq) {
                    let inclusive = self.eat(TokenKind::DotDotEq);
                    if !inclusive {
                        self.expect(TokenKind::DotDot, "expected '..' or '..=' in range pattern")?;
                    }
                    if !self.check(TokenKind::Num) {
                        return Err(FrontendError {
                            pos: self.pos(),
                            message: "expected integer literal after '..' in range pattern"
                                .to_string(),
                        });
                    }
                    let end_text = self.advance().text;
                    let start = parse_i64_pattern_bound(&num_text)?;
                    let end = parse_i64_pattern_bound(&end_text)?;
                    return Ok(MatchPattern::IntRange(IntRangePattern {
                        start,
                        end,
                        inclusive,
                    }));
                }
                // Not a range — put the number back by returning an error explaining
                // plain numeric patterns aren't supported outside ranges.
                return Err(FrontendError {
                    pos: self.pos(),
                    message: format!(
                        "plain integer literal '{}' is not a valid match pattern; use a range like {}..={} or an ADT pattern",
                        num_text, num_text, num_text
                    ),
                });
            }
        }
        if self.check(TokenKind::Ident) {
            let adt_name = self.expect_symbol()?;
            self.expect(TokenKind::PathSep, "expected '::' in enum match pattern")?;
            let variant_name = self.expect_symbol()?;
            let items = if self.eat(TokenKind::LParen) {
                self.parse_adt_match_pattern_items()?
            } else {
                Vec::new()
            };
            return Ok(MatchPattern::Adt(AdtMatchPattern {
                adt_name,
                variant_name,
                items,
            }));
        }
        Err(FrontendError {
            pos: self.pos(),
            message: "expected match pattern: N|F|T|S | _ | Type::Variant | int..int | pat | pat"
                .to_string(),
        })
    }

    fn parse_adt_match_pattern_items(&mut self) -> Result<Vec<AdtPatternItem>, FrontendError> {
        let mut items = Vec::new();
        if self.check(TokenKind::RParen) {
            return Err(FrontendError {
                pos: self.pos(),
                message: "enum match pattern payload cannot be empty parentheses; omit '()' for unit variant".to_string(),
            });
        }
        loop {
            if self.eat(TokenKind::Underscore) {
                items.push(AdtPatternItem::Discard);
            } else if self.eat(TokenKind::KwRef) {
                // M9.5 Wave B: `ref x` — borrow binding in ADT patterns.
                items.push(AdtPatternItem::Bind {
                    name: self.expect_symbol()?,
                    capture: CaptureMode::Borrow,
                });
            } else if self.check(TokenKind::Ident) {
                items.push(AdtPatternItem::Bind {
                    name: self.expect_symbol()?,
                    capture: CaptureMode::Move,
                });
            } else {
                return Err(FrontendError {
                    pos: self.pos(),
                    message: "enum match payload patterns currently support name/ref name/_ items"
                        .to_string(),
                });
            }
            if self.eat(TokenKind::Comma) {
                if self.check(TokenKind::RParen) {
                    break;
                }
                continue;
            }
            break;
        }
        self.expect(
            TokenKind::RParen,
            "expected ')' after enum match pattern payload",
        )?;
        Ok(items)
    }

    fn parse_match_guard_opt(&mut self) -> Result<Option<ExprId>, FrontendError> {
        if self.eat(TokenKind::KwIf) {
            return Ok(Some(self.parse_expr()?));
        }
        Ok(None)
    }

    fn parse_loop_expr_after_kw_loop(&mut self) -> Result<ExprId, FrontendError> {
        let body = self.parse_block()?;
        Ok(self.arena.alloc_expr(Expr::Loop(LoopExpr { body })))
    }

    fn parse_value_block(&mut self) -> Result<BlockExpr, FrontendError> {
        self.expect(TokenKind::LBrace, "expected '{'")?;
        let mut statements = Vec::new();

        loop {
            if self.check(TokenKind::RBrace) {
                return Err(FrontendError {
                    pos: self.pos(),
                    message: "value-producing block requires trailing value expression before '}'"
                        .to_string(),
                });
            }

            if self.starts_stmt_only_in_block_expr() {
                statements.push(self.parse_stmt()?);
                continue;
            }

            if self.check(TokenKind::KwGuard) || self.check(TokenKind::KwReturn) {
                return Err(FrontendError {
                    pos: self.pos(),
                    message:
                        "value-producing block currently supports only const-bindings, let-bindings, discard binds, and expression statements before the tail value"
                            .to_string(),
                });
            }

            let expr = self.parse_expr()?;
            if self.eat(TokenKind::Semi) {
                statements.push(self.arena.alloc_stmt(Stmt::Expr(expr)));
                continue;
            }

            self.expect(
                TokenKind::RBrace,
                "expected '}' after value-producing block",
            )?;
            return Ok(BlockExpr {
                statements,
                tail: expr,
            });
        }
    }

    fn parse_type(&mut self) -> Result<Type, FrontendError> {
        // Check if the next token is a quad-value token (T/F/S/N) that is
        // actually a type parameter name in the current scope. These lex as
        // QuadT/QuadF/QuadS/QuadN rather than Ident.
        {
            let i = self.next_non_layout_idx();
            if let Some(tok) = self.tokens.get(i) {
                if matches!(
                    tok.kind,
                    TokenKind::QuadT | TokenKind::QuadF | TokenKind::QuadS | TokenKind::QuadN
                ) {
                    let name = tok.text.clone();
                    let candidate = self.arena.intern_symbol(&name);
                    if self.type_param_scope.contains(&candidate) {
                        self.idx = i + 1;
                        return Ok(Type::TypeVar(candidate));
                    }
                }
            }
        }
        let base = if self.eat(TokenKind::LParen) {
            self.parse_paren_type_or_tuple()?
        } else if self.check(TokenKind::Ident) {
            let t = self.tokens[self.next_non_layout_idx()].text.clone();
            if t == "Self" {
                let _ = self.advance();
                if let Some(self_ty) = &self.self_type_scope {
                    self_ty.clone()
                } else {
                    Type::Record(self.arena.intern_symbol("Self"))
                }
            } else if t == "qvec" {
                let _ = self.advance();
                if self.eat(TokenKind::LBracket) || self.eat(TokenKind::LParen) {
                    let n = if self.check(TokenKind::Num) {
                        self.advance().text.parse::<usize>().unwrap_or(32)
                    } else {
                        32
                    };
                    let _ = self.eat(TokenKind::RBracket) || self.eat(TokenKind::RParen);
                    Type::QVec(n)
                } else {
                    Type::QVec(32)
                }
            } else if t == "Option" {
                let lookahead = self.next_non_layout_idx_from(self.next_non_layout_idx() + 1);
                if self
                    .tokens
                    .get(lookahead)
                    .map(|tok| tok.kind == TokenKind::LParen)
                    .unwrap_or(false)
                {
                    let _ = self.advance();
                    self.expect(TokenKind::LParen, "expected '(' after Option type name")?;
                    let item = self.parse_type()?;
                    self.expect(TokenKind::RParen, "expected ')' after Option type argument")?;
                    Type::Option(Box::new(item))
                } else {
                    let record_name = self.expect_symbol()?;
                    Type::Record(record_name)
                }
            } else if t == "Result" {
                let lookahead = self.next_non_layout_idx_from(self.next_non_layout_idx() + 1);
                if self
                    .tokens
                    .get(lookahead)
                    .map(|tok| tok.kind == TokenKind::LParen)
                    .unwrap_or(false)
                {
                    let _ = self.advance();
                    self.expect(TokenKind::LParen, "expected '(' after Result type name")?;
                    let ok_ty = self.parse_type()?;
                    self.expect(
                        TokenKind::Comma,
                        "expected ',' between Result type arguments",
                    )?;
                    let err_ty = self.parse_type()?;
                    self.expect(
                        TokenKind::RParen,
                        "expected ')' after Result type arguments",
                    )?;
                    Type::Result(Box::new(ok_ty), Box::new(err_ty))
                } else {
                    let record_name = self.expect_symbol()?;
                    Type::Record(record_name)
                }
            } else if t == "Sequence" {
                let lookahead = self.next_non_layout_idx_from(self.next_non_layout_idx() + 1);
                if self
                    .tokens
                    .get(lookahead)
                    .map(|tok| tok.kind == TokenKind::LParen)
                    .unwrap_or(false)
                {
                    let _ = self.advance();
                    self.expect(TokenKind::LParen, "expected '(' after Sequence type name")?;
                    let item = self.parse_type()?;
                    self.expect(
                        TokenKind::RParen,
                        "expected ')' after Sequence type argument",
                    )?;
                    Type::Sequence(SequenceType {
                        family: SequenceCollectionFamily::OrderedSequence,
                        item: Box::new(item),
                    })
                } else {
                    let record_name = self.expect_symbol()?;
                    Type::Record(record_name)
                }
            } else if t == "Map" {
                let lookahead = self.next_non_layout_idx_from(self.next_non_layout_idx() + 1);
                if self
                    .tokens
                    .get(lookahead)
                    .map(|tok| tok.kind == TokenKind::LParen)
                    .unwrap_or(false)
                {
                    let _ = self.advance();
                    self.expect(TokenKind::LParen, "expected '(' after Map type name")?;
                    let key = self.parse_type()?;
                    self.expect(
                        TokenKind::Comma,
                        "expected ',' between Map key and value types",
                    )?;
                    let val = self.parse_type()?;
                    self.expect(TokenKind::RParen, "expected ')' after Map type arguments")?;
                    Type::Map(MapType {
                        key: Box::new(key),
                        val: Box::new(val),
                    })
                } else {
                    let record_name = self.expect_symbol()?;
                    Type::Record(record_name)
                }
            } else if t == "Closure" {
                let lookahead = self.next_non_layout_idx_from(self.next_non_layout_idx() + 1);
                if self
                    .tokens
                    .get(lookahead)
                    .map(|tok| tok.kind == TokenKind::LParen)
                    .unwrap_or(false)
                {
                    let _ = self.advance();
                    self.expect(TokenKind::LParen, "expected '(' after Closure type name")?;
                    let param_ty = self.parse_type()?;
                    self.expect(
                        TokenKind::Implies,
                        "expected '->' between Closure parameter and return types",
                    )?;
                    let ret_ty = self.parse_type()?;
                    self.expect(TokenKind::RParen, "expected ')' after Closure type")?;
                    Type::Closure(crate::types::ClosureType {
                        family: ClosureValueFamily::UnaryDirect,
                        capture: ClosureCapturePolicy::Immutable,
                        param: Box::new(param_ty),
                        ret: Box::new(ret_ty),
                    })
                } else {
                    let record_name = self.expect_symbol()?;
                    Type::Record(record_name)
                }
            } else if t == "text" {
                let _ = self.advance();
                Type::Text
            } else {
                let record_name = self.expect_symbol()?;
                // If the name matches a type parameter in scope, emit TypeVar
                // rather than a nominal Record reference.
                if self.type_param_scope.contains(&record_name) {
                    Type::TypeVar(record_name)
                } else {
                    Type::Record(record_name)
                }
            }
        } else if self.eat(TokenKind::TyQuad) {
            Type::Quad
        } else if self.eat(TokenKind::TyBool) {
            Type::Bool
        } else if self.eat(TokenKind::TyI32) {
            Type::I32
        } else if self.eat(TokenKind::TyU32) {
            Type::U32
        } else if self.eat(TokenKind::TyFx) {
            Type::Fx
        } else if self.eat(TokenKind::TyF64) {
            self.require_f64_feature("type 'f64' is disabled by profile policy")?;
            Type::F64
        } else {
            return Err(FrontendError {
                pos: self.pos(),
                message: "expected type".to_string(),
            });
        };
        self.parse_optional_measure_annotation(base)
    }

    fn parse_optional_measure_annotation(&mut self, base: Type) -> Result<Type, FrontendError> {
        if !self.eat(TokenKind::LBracket) {
            return Ok(base);
        }
        if !base.is_core_numeric_scalar() {
            return Err(FrontendError {
                pos: self.pos(),
                message: "unit annotation is allowed only on i32, u32, f64, or fx in v0"
                    .to_string(),
            });
        }
        let unit = self.expect_symbol()?;
        self.expect(TokenKind::RBracket, "expected ']' after unit annotation")?;
        Ok(Type::Measured(Box::new(base), unit))
    }

    fn parse_paren_type_or_tuple(&mut self) -> Result<Type, FrontendError> {
        if self.check(TokenKind::RParen) {
            return Err(FrontendError {
                pos: self.pos(),
                message: "empty tuple type is not supported in v0".to_string(),
            });
        }
        let first = self.parse_type()?;
        if !self.eat(TokenKind::Comma) {
            self.expect(TokenKind::RParen, "expected ')' after parenthesized type")?;
            return Ok(first);
        }

        let mut items = vec![first, self.parse_type()?];
        while self.eat(TokenKind::Comma) {
            if self.check(TokenKind::RParen) {
                break;
            }
            items.push(self.parse_type()?);
        }
        self.expect(TokenKind::RParen, "expected ')' after tuple type")?;
        Ok(Type::Tuple(items))
    }

    fn parse_logos_program(&mut self) -> Result<LogosProgram, FrontendError> {
        self.require_logos_surface("Logos surface is disabled by profile policy")?;
        let mut out = LogosProgram::default();
        let mut errors: Vec<FrontendError> = Vec::new();
        while self.idx < self.tokens.len() {
            self.skip_newlines_raw();
            if self.idx >= self.tokens.len() {
                break;
            }
            if self.check_raw(TokenKind::KwSystem) {
                match self.parse_logos_system() {
                    Ok(system) => out.system = Some(system),
                    Err(e) => {
                        errors.push(e);
                        self.recover_logos_anchor();
                    }
                }
                continue;
            }
            if self.check_raw(TokenKind::KwEntity) {
                match self.parse_logos_entity() {
                    Ok(entity) => out.entities.push(entity),
                    Err(e) => {
                        errors.push(e);
                        self.recover_logos_anchor();
                    }
                }
                continue;
            }
            if self.check_raw(TokenKind::KwLaw) {
                match self.parse_logos_law() {
                    Ok(law) => out.laws.push(law),
                    Err(e) => {
                        errors.push(e);
                        self.recover_logos_anchor();
                    }
                }
                continue;
            }
            if self.check_raw(TokenKind::KwImport)
                || self.check_raw(TokenKind::KwPulse)
                || self.check_raw(TokenKind::KwProfile)
            {
                self.require_legacy_compatibility(
                    "legacy Logos directives require legacy compatibility mode",
                )?;
                while !self.check_raw(TokenKind::Newline) && self.idx < self.tokens.len() {
                    self.idx += 1;
                }
                self.eat_raw(TokenKind::Newline);
                continue;
            }
            let mut msg = "expected Logos declaration".to_string();
            if let Some(tok) = self.tokens.get(self.idx) {
                if tok.kind == TokenKind::Ident {
                    if let Some(s) = suggest_closest_case_insensitive(
                        &tok.text,
                        &["System", "Entity", "Law", "Import", "Pulse", "Profile"],
                        3,
                    ) {
                        msg.push_str(&format!("\nhelp: did you mean '{}'?", s));
                    }
                }
            }
            errors.push(self.error_at_current(&msg, "E0200"));
            self.recover_logos_anchor();
        }
        if !errors.is_empty() {
            return Err(self.merge_logos_errors(errors));
        }
        out.laws.sort_by(|a, b| b.priority.cmp(&a.priority));
        Ok(out)
    }

    fn recover_logos_anchor(&mut self) {
        while self.idx < self.tokens.len() {
            let k = self.tokens[self.idx].kind;
            if matches!(
                k,
                TokenKind::Newline
                    | TokenKind::Dedent
                    | TokenKind::KwSystem
                    | TokenKind::KwEntity
                    | TokenKind::KwLaw
            ) {
                if matches!(k, TokenKind::Newline | TokenKind::Dedent) {
                    self.idx += 1;
                }
                break;
            }
            self.idx += 1;
        }
    }

    fn merge_logos_errors(&self, errors: Vec<FrontendError>) -> FrontendError {
        let pos = errors.first().map(|e| e.pos).unwrap_or(0);
        let msgs: Vec<String> = errors.into_iter().map(|e| e.message).collect();
        FrontendError {
            pos,
            message: format_multiple_parser_errors("E0200", &msgs),
        }
    }

    fn parse_logos_system(&mut self) -> Result<LogosSystem, FrontendError> {
        let kw = self.expect_raw(TokenKind::KwSystem, "expected 'System'", "E0201")?;
        let name = self.expect_raw_ident()?;
        let mut params = Vec::new();
        if self.eat_raw(TokenKind::LParen) {
            if !self.check_raw(TokenKind::RParen) {
                loop {
                    let pname = self.expect_raw_ident()?;
                    self.expect_raw(TokenKind::Assign, "expected '='", "E0202")?;
                    let pval = self.expect_raw_ident_or_num()?;
                    params.push((pname, pval));
                    if self.eat_raw(TokenKind::Comma) {
                        continue;
                    }
                    break;
                }
            }
            self.expect_raw(TokenKind::RParen, "expected ')'", "E0203")?;
        }
        self.eat_raw(TokenKind::Colon);
        self.eat_raw(TokenKind::Newline);
        Ok(LogosSystem {
            name,
            params,
            mark: kw.mark,
        })
    }

    fn parse_logos_entity(&mut self) -> Result<LogosEntity, FrontendError> {
        let kw = self.expect_raw(TokenKind::KwEntity, "expected 'Entity'", "E0210")?;
        let name = self.expect_raw_ident()?;
        self.expect_raw(TokenKind::Colon, "expected ':'", "E0211")?;
        self.expect_raw(TokenKind::Newline, "expected newline", "E0212")?;
        self.expect_raw(TokenKind::Indent, "expected INDENT", "E0213")?;

        let mut fields = Vec::new();
        loop {
            self.skip_newlines_raw();
            if self.eat_raw(TokenKind::Dedent) {
                break;
            }
            let kind_tok =
                self.expect_raw(TokenKind::Ident, "expected 'state' or 'prop'", "E0214")?;
            let field_kind = match kind_tok.text.as_str() {
                "state" => LogosEntityFieldKind::State,
                "prop" => LogosEntityFieldKind::Prop,
                _ => {
                    let mut msg = "expected 'state' or 'prop'".to_string();
                    if let Some(s) =
                        suggest_closest_case_insensitive(&kind_tok.text, &["state", "prop"], 3)
                    {
                        msg.push_str(&format!("\nhelp: did you mean '{}'?", s));
                    }
                    return Err(self.error_at_token(&kind_tok, &msg, "E0215"));
                }
            };
            let field_name = self.expect_raw_ident()?;
            self.expect_raw(TokenKind::Colon, "expected ':'", "E0216")?;
            let ty = self.parse_type_raw()?;
            fields.push(LogosEntityField {
                kind: field_kind,
                name: field_name,
                ty,
                mark: kind_tok.mark,
            });
            self.eat_raw(TokenKind::Newline);
        }

        Ok(LogosEntity {
            name,
            fields,
            mark: kw.mark,
        })
    }

    fn parse_logos_law(&mut self) -> Result<LogosLaw, FrontendError> {
        let kw = self.expect_raw(TokenKind::KwLaw, "expected 'Law'", "E0220")?;
        let name_tok = self.expect_raw(TokenKind::String, "expected law name", "E0221")?;
        let name = name_tok.text.trim_matches('"').to_string();
        let mut priority = 0u32;
        if self.eat_raw(TokenKind::LBracket) {
            let p_kw = self.expect_raw(TokenKind::Ident, "expected 'priority'", "E0222")?;
            if p_kw.text != "priority" {
                return Err(self.error_at_token(&p_kw, "expected 'priority'", "E0223"));
            }
            let num = self.expect_raw(TokenKind::Num, "expected priority number", "E0224")?;
            priority = num
                .text
                .parse::<u32>()
                .map_err(|_| self.error_at_token(&num, "invalid priority value", "E0225"))?;
            self.expect_raw(TokenKind::RBracket, "expected ']'", "E0226")?;
        }
        self.expect_raw(TokenKind::Colon, "expected ':'", "E0227")?;
        self.expect_raw(TokenKind::Newline, "expected newline", "E0228")?;
        self.expect_raw(TokenKind::Indent, "expected INDENT", "E0229")?;

        let mut whens = Vec::new();
        loop {
            self.skip_newlines_raw();
            if self.eat_raw(TokenKind::Dedent) {
                break;
            }
            let when_tok = self.expect_raw(TokenKind::KwWhen, "expected 'When'", "E0230")?;
            let condition_tokens = self.collect_until_raw(TokenKind::Implies)?;
            if condition_tokens.is_empty() {
                return Err(self.error_at_token(&when_tok, "empty When condition", "E0231"));
            }
            self.expect_raw(TokenKind::Implies, "expected '->'", "E0232")?;
            let effect_tokens = if self.eat_raw(TokenKind::Newline) {
                self.skip_newlines_raw();
                self.collect_until_newline_or_dedent()
            } else {
                self.collect_until_newline_or_dedent()
            };
            if effect_tokens.is_empty() {
                return Err(self.error_at_token(&when_tok, "empty When effect", "E0233"));
            }
            let condition = self.join_token_text(&condition_tokens);
            let effect = self.join_token_text(&effect_tokens);
            whens.push(LogosWhen {
                condition,
                effect,
                mark: when_tok.mark,
            });
            self.eat_raw(TokenKind::Newline);
        }

        Ok(LogosLaw {
            name,
            priority,
            whens,
            mark: kw.mark,
        })
    }

    fn parse_type_raw(&mut self) -> Result<Type, FrontendError> {
        let base = if self.check_raw(TokenKind::Ident) {
            let t = self.tokens[self.idx].text.clone();
            if t == "qvec" {
                self.idx += 1;
                Type::QVec(32)
            } else {
                return Err(self.error_at_current("expected type", "E0234"));
            }
        } else if self.eat_raw(TokenKind::TyQuad) {
            Type::Quad
        } else if self.eat_raw(TokenKind::TyBool) {
            Type::Bool
        } else if self.eat_raw(TokenKind::TyI32) {
            Type::I32
        } else if self.eat_raw(TokenKind::TyU32) {
            Type::U32
        } else if self.eat_raw(TokenKind::TyFx) {
            Type::Fx
        } else if self.eat_raw(TokenKind::TyF64) {
            self.require_f64_feature("type 'f64' is disabled by profile policy")?;
            Type::F64
        } else {
            return Err(self.error_at_current("expected type", "E0234"));
        };
        self.parse_optional_measure_annotation_raw(base)
    }

    fn parse_optional_measure_annotation_raw(&mut self, base: Type) -> Result<Type, FrontendError> {
        if !self.eat_raw(TokenKind::LBracket) {
            return Ok(base);
        }
        if !base.is_core_numeric_scalar() {
            return Err(self.error_at_current(
                "unit annotation is allowed only on i32, u32, f64, or fx in v0",
                "E0234",
            ));
        }
        if !self.check_raw(TokenKind::Ident) {
            return Err(self.error_at_current("expected unit symbol", "E0234"));
        }
        let unit = self
            .arena
            .intern_symbol(&self.tokens[self.idx].text.clone());
        self.idx += 1;
        self.expect_raw(TokenKind::RBracket, "expected ']'", "E0234")?;
        Ok(Type::Measured(Box::new(base), unit))
    }

    fn collect_until_raw(&mut self, stop: TokenKind) -> Result<Vec<Token>, FrontendError> {
        let mut out = Vec::new();
        let mut paren = 0usize;
        while self.idx < self.tokens.len() {
            let t = self.tokens[self.idx].clone();
            if t.kind == TokenKind::LParen {
                paren += 1;
            } else if t.kind == TokenKind::RParen {
                paren = paren.saturating_sub(1);
            }
            if paren == 0 && t.kind == stop {
                break;
            }
            if t.kind == TokenKind::Newline || t.kind == TokenKind::Dedent {
                return Err(self.error_at_token(
                    &t,
                    "unexpected line break in expression",
                    "E0235",
                ));
            }
            out.push(t);
            self.idx += 1;
        }
        Ok(out)
    }

    fn collect_until_newline_or_dedent(&mut self) -> Vec<Token> {
        let mut out = Vec::new();
        while self.idx < self.tokens.len() {
            let t = self.tokens[self.idx].clone();
            if matches!(t.kind, TokenKind::Newline | TokenKind::Dedent) {
                break;
            }
            out.push(t);
            self.idx += 1;
        }
        out
    }

    fn join_token_text(&self, toks: &[Token]) -> String {
        toks.iter()
            .map(|t| t.text.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn check_raw(&self, kind: TokenKind) -> bool {
        self.tokens
            .get(self.idx)
            .map(|t| t.kind == kind)
            .unwrap_or(false)
    }

    fn eat_raw(&mut self, kind: TokenKind) -> bool {
        if self.check_raw(kind) {
            self.idx += 1;
            true
        } else {
            false
        }
    }

    fn expect_raw(
        &mut self,
        kind: TokenKind,
        msg: &str,
        code: &str,
    ) -> Result<Token, FrontendError> {
        if self.check_raw(kind) {
            let t = self.tokens[self.idx].clone();
            self.idx += 1;
            Ok(t)
        } else {
            Err(self.error_at_current(msg, code))
        }
    }

    fn expect_raw_ident(&mut self) -> Result<String, FrontendError> {
        let tok = self.expect_raw(TokenKind::Ident, "expected identifier", "E0236")?;
        Ok(tok.text)
    }

    fn expect_raw_ident_or_num(&mut self) -> Result<String, FrontendError> {
        if self.check_raw(TokenKind::Ident)
            || self.check_raw(TokenKind::Num)
            || self.check_raw(TokenKind::String)
        {
            let t = self.tokens[self.idx].clone();
            self.idx += 1;
            return Ok(t.text);
        }
        Err(self.error_at_current("expected identifier/number", "E0237"))
    }

    fn skip_newlines_raw(&mut self) {
        while self.eat_raw(TokenKind::Newline) {}
    }

    fn require_f64_feature(&self, message: &str) -> Result<(), FrontendError> {
        if self.policy.profile.features.allow_f64_math {
            Ok(())
        } else {
            Err(FrontendError::policy_violation(self.pos(), message))
        }
    }

    fn parse_numeric_literal_expr(&mut self, text: &str) -> Result<ExprId, FrontendError> {
        let (core, suffix) = split_numeric_suffix(text);
        let literal = match suffix {
            Some("i32") => NumericLiteral::I32(parse_i32_literal(core)?),
            Some("u32") => NumericLiteral::U32(parse_u32_literal(core)?),
            Some("f64") => {
                self.require_f64_feature("f64 literals are disabled by profile policy")?;
                NumericLiteral::F64(parse_decimal_f64_literal(core, "f64")?)
            }
            Some("fx") => NumericLiteral::Fx(parse_decimal_f64_literal(core, "fx")?),
            Some(_) => {
                return Err(FrontendError {
                    pos: self.pos(),
                    message: "unsupported numeric literal suffix".to_string(),
                });
            }
            None if core.contains('.') => {
                self.require_f64_feature("f64 literals are disabled by profile policy")?;
                NumericLiteral::F64(parse_decimal_f64_literal(core, "f64")?)
            }
            None => NumericLiteral::I32(parse_i32_literal(core)?),
        };
        Ok(self.arena.alloc_expr(Expr::NumericLiteral(literal)))
    }

    fn require_logos_surface(&self, message: &str) -> Result<(), FrontendError> {
        if self.policy.profile.features.allow_logos_surface {
            Ok(())
        } else {
            Err(FrontendError::policy_violation(self.pos(), message))
        }
    }

    fn require_schema_surface(&self, message: &str) -> Result<(), FrontendError> {
        if self.policy.profile.features.allow_schema_surface {
            Ok(())
        } else {
            Err(FrontendError::policy_violation(self.pos(), message))
        }
    }

    fn require_legacy_compatibility(&self, message: &str) -> Result<(), FrontendError> {
        if self.policy.profile.compatibility == CompatibilityMode::LegacySupport {
            Ok(())
        } else {
            Err(FrontendError::policy_violation(self.pos(), message))
        }
    }

    fn error_at_current(&self, msg: &str, code: &str) -> FrontendError {
        if let Some(tok) = self.tokens.get(self.idx) {
            self.error_at_token(tok, msg, code)
        } else {
            FrontendError {
                pos: self.pos(),
                message: format!("error[{code}]: {msg}"),
            }
        }
    }

    fn error_at_token(&self, tok: &Token, msg: &str, code: &str) -> FrontendError {
        let line = tok.mark.line.max(1);
        let col = tok.mark.col.max(1);
        let mut sm = SourceMap::new();
        let fid = sm.add_file("<input>", &self.source);
        let src_line = sm.line(fid, line).unwrap_or_default();
        FrontendError {
            pos: tok.pos,
            message: format_parser_error_at_input(code, msg, line, col, src_line),
        }
    }

    fn pos(&self) -> usize {
        self.tokens.get(self.idx).map(|t| t.pos).unwrap_or(0)
    }

    fn is_layout(kind: TokenKind) -> bool {
        matches!(
            kind,
            TokenKind::Newline | TokenKind::Indent | TokenKind::Dedent
        )
    }

    fn next_non_layout_idx(&self) -> usize {
        let mut i = self.idx;
        while i < self.tokens.len() && Self::is_layout(self.tokens[i].kind) {
            i += 1;
        }
        i
    }

    fn next_non_layout_idx_from(&self, mut i: usize) -> usize {
        while i < self.tokens.len() && Self::is_layout(self.tokens[i].kind) {
            i += 1;
        }
        i
    }

    fn check(&self, kind: TokenKind) -> bool {
        let i = self.next_non_layout_idx();
        self.tokens.get(i).map(|t| t.kind == kind).unwrap_or(false)
    }

    fn eat_ident_text(&mut self, text: &str) -> bool {
        let i = self.next_non_layout_idx();
        if self
            .tokens
            .get(i)
            .map(|t| t.kind == TokenKind::Ident && t.text == text)
            .unwrap_or(false)
        {
            self.idx = i + 1;
            true
        } else {
            false
        }
    }

    fn eat(&mut self, kind: TokenKind) -> bool {
        let i = self.next_non_layout_idx();
        if self.tokens.get(i).map(|t| t.kind == kind).unwrap_or(false) {
            self.idx = i + 1;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, kind: TokenKind, msg: &str) -> Result<(), FrontendError> {
        if self.eat(kind) {
            Ok(())
        } else {
            Err(FrontendError {
                pos: self.pos(),
                message: msg.to_string(),
            })
        }
    }

    /// Accept an identifier as a type parameter name.
    ///
    /// Extends `expect_symbol` to also accept quad-value tokens (`T`, `F`,
    /// `S`, `N`) since those single letters lex as `QuadT/QuadF/QuadS/QuadN`
    /// rather than `Ident`, but are conventional type-parameter names.
    fn expect_type_param_name(&mut self) -> Result<SymbolId, FrontendError> {
        let i = self.next_non_layout_idx();
        let is_type_param_name = self
            .tokens
            .get(i)
            .map(|t| {
                matches!(
                    t.kind,
                    TokenKind::Ident
                        | TokenKind::QuadT
                        | TokenKind::QuadF
                        | TokenKind::QuadS
                        | TokenKind::QuadN
                )
            })
            .unwrap_or(false);
        if is_type_param_name {
            let name = self.advance().text;
            Ok(self.arena.intern_symbol(&name))
        } else {
            Err(FrontendError {
                pos: self.pos(),
                message: "expected type parameter name".to_string(),
            })
        }
    }

    fn expect_symbol(&mut self) -> Result<SymbolId, FrontendError> {
        if self.check(TokenKind::Ident) {
            let name = self.advance().text;
            Ok(self.arena.intern_symbol(&name))
        } else {
            Err(FrontendError {
                pos: self.pos(),
                message: "expected identifier".to_string(),
            })
        }
    }

    fn expect_string_literal_text(&mut self, msg: &str) -> Result<String, FrontendError> {
        if self.check(TokenKind::String) {
            let token = self.advance().text;
            Ok(token.trim_matches('"').to_string())
        } else {
            Err(FrontendError {
                pos: self.pos(),
                message: msg.to_string(),
            })
        }
    }

    fn advance(&mut self) -> Token {
        let i = self.next_non_layout_idx();
        let t = self.tokens[i].clone();
        self.idx = i + 1;
        t
    }

    /// Peek at the current non-layout token without consuming it.
    fn peek(&self) -> Token {
        let i = self.next_non_layout_idx();
        self.tokens.get(i).cloned().unwrap_or(Token {
            kind: TokenKind::Dedent,
            text: String::new(),
            pos: 0,
            mark: Default::default(),
        })
    }
}

/// Parse a plain integer literal as an i64 range bound.
/// Only decimal and hex (`0x`) forms are accepted; no suffixes, no decimals.
fn parse_i64_pattern_bound(text: &str) -> Result<i64, FrontendError> {
    if text.contains('.') {
        return Err(FrontendError {
            pos: 0,
            message: "range pattern bound must be an integer literal, not a float".to_string(),
        });
    }
    let (core, suffix) = split_numeric_suffix(text);
    if suffix.is_some() {
        return Err(FrontendError {
            pos: 0,
            message: "range pattern bound does not accept a type suffix; use a plain integer"
                .to_string(),
        });
    }
    if let Some(hex) = core.strip_prefix("0x").or_else(|| core.strip_prefix("0X")) {
        let digits = strip_digit_separators(hex);
        return i64::from_str_radix(&digits, 16).map_err(|_| FrontendError {
            pos: 0,
            message: "invalid hexadecimal range pattern bound".to_string(),
        });
    }
    let digits = strip_digit_separators(core);
    digits.parse::<i64>().map_err(|_| FrontendError {
        pos: 0,
        message: format!("invalid integer range pattern bound '{}'", text),
    })
}

fn split_numeric_suffix(text: &str) -> (&str, Option<&str>) {
    for suffix in ["i32", "u32", "f64", "fx"] {
        if let Some(core) = text.strip_suffix(suffix) {
            return (core, Some(suffix));
        }
    }
    (text, None)
}

fn strip_digit_separators(text: &str) -> String {
    text.chars().filter(|ch| *ch != '_').collect()
}

fn parse_i32_literal(text: &str) -> Result<i32, FrontendError> {
    if text.contains('.') {
        return Err(FrontendError {
            pos: 0,
            message: "i32 literal cannot contain decimal point".to_string(),
        });
    }
    if let Some(hex) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
        let digits = strip_digit_separators(hex);
        return i32::from_str_radix(&digits, 16).map_err(|_| FrontendError {
            pos: 0,
            message: "invalid i32 hexadecimal literal".to_string(),
        });
    }
    let digits = strip_digit_separators(text);
    digits.parse::<i32>().map_err(|_| FrontendError {
        pos: 0,
        message: "invalid i32 literal".to_string(),
    })
}

fn parse_u32_literal(text: &str) -> Result<u32, FrontendError> {
    if text.contains('.') {
        return Err(FrontendError {
            pos: 0,
            message: "u32 literal cannot contain decimal point".to_string(),
        });
    }
    if let Some(hex) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
        let digits = strip_digit_separators(hex);
        return u32::from_str_radix(&digits, 16).map_err(|_| FrontendError {
            pos: 0,
            message: "invalid u32 hexadecimal literal".to_string(),
        });
    }
    let digits = strip_digit_separators(text);
    digits.parse::<u32>().map_err(|_| FrontendError {
        pos: 0,
        message: "invalid u32 literal".to_string(),
    })
}

fn parse_decimal_f64_literal(text: &str, kind: &str) -> Result<f64, FrontendError> {
    if text.starts_with("0x") || text.starts_with("0X") {
        return Err(FrontendError {
            pos: 0,
            message: format!("{kind} literal currently requires decimal form"),
        });
    }
    let digits = strip_digit_separators(text);
    digits.parse::<f64>().map_err(|_| FrontendError {
        pos: 0,
        message: format!("invalid {kind} literal"),
    })
}

fn parse_schema_version_literal(text: &str) -> Result<u32, String> {
    let (core, suffix) = split_numeric_suffix(text);
    if suffix.is_some() || core.contains('.') || core.starts_with("0x") || core.starts_with("0X") {
        return Err(
            "schema version marker currently requires unsuffixed decimal integer".to_string(),
        );
    }
    let digits = strip_digit_separators(core);
    let value = digits
        .parse::<u32>()
        .map_err(|_| "invalid schema version marker".to_string())?;
    if value == 0 {
        return Err("schema version marker must be positive".to_string());
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FrontendErrorKind;
    use sm_profile::ParserProfile;

    #[test]
    fn rustlike_parser_smoke() {
        let src = r#"
fn idq(q: quad) -> quad { return q; }
fn main() { let x: quad = idq(T); return; }
"#;
        let a = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("frontend rustlike");
        assert_eq!(a.functions.len(), 2);
    }

    #[test]
    fn rustlike_parser_accepts_expression_bodied_function() {
        let src = r#"
fn idq(q: quad) -> quad = q;
fn main() { return; }
        "#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("expression-bodied function should parse");
        let func = &program.functions[0];
        let Stmt::Return(Some(value)) = program.arena.stmt(func.body[0]) else {
            panic!("expected desugared return statement");
        };
        assert!(matches!(program.arena.expr(*value), Expr::Var(_)));
    }

    #[test]
    fn rustlike_parser_accepts_top_level_namespace_import() {
        let src = r#"
Import "helper.sm"

fn main() { return; }
        "#;

        let program =
            parse_rustlike_with_profile(src, &ParserProfile::foundation_default()).expect("parse");
        assert_eq!(program.imports.len(), 1);
        let import = &program.imports[0];
        assert_eq!(import.spec, "helper.sm");
        assert_eq!(import.alias, None);
        assert!(!import.reexport);
        assert!(!import.wildcard);
        assert!(import.select_items.is_empty());
    }

    #[test]
    fn rustlike_parser_preserves_import_alias_and_select_items() {
        let src = r#"
Import "helper.sm" as Helpers { Foo, Bar as Baz }

fn main() { return; }
        "#;

        let program =
            parse_rustlike_with_profile(src, &ParserProfile::foundation_default()).expect("parse");
        assert_eq!(program.imports.len(), 1);
        let import = &program.imports[0];
        assert_eq!(import.spec, "helper.sm");
        assert_eq!(
            import
                .alias
                .map(|sym| program.arena.symbol_name(sym).to_string()),
            Some("Helpers".to_string())
        );
        assert_eq!(import.select_items.len(), 2);
        assert_eq!(
            program.arena.symbol_name(import.select_items[0].name),
            "Foo"
        );
        assert_eq!(import.select_items[0].alias, None);
        assert_eq!(
            program.arena.symbol_name(import.select_items[1].name),
            "Bar"
        );
        assert_eq!(
            import.select_items[1]
                .alias
                .map(|sym| program.arena.symbol_name(sym).to_string()),
            Some("Baz".to_string())
        );
    }

    #[test]
    fn rustlike_parser_accepts_function_requires_clause() {
        let src = r#"
record DecisionContext {
    camera: quad,
    quality: f64,
}

fn decide(ctx: DecisionContext) -> quad requires(ctx.camera == T) {
    return ctx.camera;
}

fn main() { return; }
        "#;

        let program =
            parse_rustlike_with_profile(src, &ParserProfile::foundation_default()).expect("parse");
        let decide = &program.functions[0];
        assert_eq!(program.arena.symbol_name(decide.name), "decide");
        assert_eq!(decide.requires.len(), 1);
        assert!(matches!(
            program.arena.expr(decide.requires[0]),
            Expr::Binary(_, BinaryOp::Eq, _)
        ));
    }

    #[test]
    fn rustlike_parser_accepts_expression_bodied_function_with_requires_clause() {
        let src = r#"
fn idq(q: quad) -> quad requires(q == T) = q;
fn main() { return; }
        "#;

        let program =
            parse_rustlike_with_profile(src, &ParserProfile::foundation_default()).expect("parse");
        let idq = &program.functions[0];
        assert_eq!(idq.requires.len(), 1);
        assert!(matches!(
            program.arena.expr(idq.requires[0]),
            Expr::Binary(_, BinaryOp::Eq, _)
        ));
    }

    #[test]
    fn rustlike_parser_accepts_function_ensures_clause() {
        let src = r#"
record DecisionContext {
    camera: quad,
    quality: f64,
}

fn decide(ctx: DecisionContext) -> quad ensures(result == ctx.camera) {
    return ctx.camera;
}

fn main() { return; }
        "#;

        let program =
            parse_rustlike_with_profile(src, &ParserProfile::foundation_default()).expect("parse");
        let decide = &program.functions[0];
        assert_eq!(program.arena.symbol_name(decide.name), "decide");
        assert_eq!(decide.ensures.len(), 1);
        assert!(matches!(
            program.arena.expr(decide.ensures[0]),
            Expr::Binary(_, BinaryOp::Eq, _)
        ));
    }

    #[test]
    fn rustlike_parser_accepts_expression_bodied_function_with_ensures_clause() {
        let src = r#"
fn idq(q: quad) -> quad ensures(result == q) = q;
fn main() { return; }
        "#;

        let program =
            parse_rustlike_with_profile(src, &ParserProfile::foundation_default()).expect("parse");
        let idq = &program.functions[0];
        assert_eq!(idq.ensures.len(), 1);
        assert!(matches!(
            program.arena.expr(idq.ensures[0]),
            Expr::Binary(_, BinaryOp::Eq, _)
        ));
    }

    #[test]
    fn rustlike_parser_accepts_function_invariant_clause() {
        let src = r#"
record DecisionContext {
    camera: quad,
    quality: f64,
}

fn decide(ctx: DecisionContext) -> quad invariant(ctx.quality == 0.75) {
    return ctx.camera;
}

fn main() { return; }
        "#;

        let program =
            parse_rustlike_with_profile(src, &ParserProfile::foundation_default()).expect("parse");
        let decide = &program.functions[0];
        assert_eq!(program.arena.symbol_name(decide.name), "decide");
        assert_eq!(decide.invariants.len(), 1);
        assert!(matches!(
            program.arena.expr(decide.invariants[0]),
            Expr::Binary(_, BinaryOp::Eq, _)
        ));
    }

    #[test]
    fn rustlike_parser_accepts_expression_bodied_function_with_invariant_clause() {
        let src = r#"
fn idq(q: quad) -> quad invariant(result == q) = q;
fn main() { return; }
        "#;

        let program =
            parse_rustlike_with_profile(src, &ParserProfile::foundation_default()).expect("parse");
        let idq = &program.functions[0];
        assert_eq!(idq.invariants.len(), 1);
        assert!(matches!(
            program.arena.expr(idq.invariants[0]),
            Expr::Binary(_, BinaryOp::Eq, _)
        ));
    }

    #[test]
    fn rustlike_parser_rejects_expression_bodied_function_without_semi() {
        let src = r#"
fn idq(q: quad) -> quad = q
fn main() { return; }
"#;

        let err = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect_err("missing semicolon must reject");
        assert!(err
            .message
            .contains("expected ';' after expression-bodied function"));
    }

    #[test]
    fn rustlike_parser_accepts_compound_assignment() {
        let src = r#"
fn main() {
    let mut total: f64 = 1.0;
    total += 2.0;
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("compound assignment should parse");
        let func = &program.functions[0];
        let Stmt::Assign { name, value } = program.arena.stmt(func.body[1]) else {
            panic!("expected compound assignment statement");
        };
        assert_eq!(program.arena.symbol_name(*name), "total");
        let Expr::Binary(lhs, BinaryOp::Add, rhs) = program.arena.expr(*value) else {
            panic!("expected desugared additive assignment");
        };
        assert!(matches!(program.arena.expr(*lhs), Expr::Var(_)));
        assert!(matches!(
            program.arena.expr(*rhs),
            Expr::NumericLiteral(NumericLiteral::F64(_))
        ));
    }

    #[test]
    fn rustlike_parser_accepts_mutable_let_binding() {
        let src = r#"
fn main() {
    let mut score: i32 = 0;
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("let mut should parse");
        let func = &program.functions[0];
        let Stmt::Let {
            name,
            is_mut,
            ty: Some(Type::I32),
            ..
        } = program.arena.stmt(func.body[0])
        else {
            panic!("expected mutable let binding");
        };
        assert_eq!(program.arena.symbol_name(*name), "score");
        assert!(*is_mut, "let mut binding should record mutability");
    }

    #[test]
    fn rustlike_parser_accepts_plain_reassignment_statement() {
        let src = r#"
fn main() {
    let mut score: i32 = 0;
    score = 1;
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("plain reassignment should parse");
        let func = &program.functions[0];
        let Stmt::Assign { name, value } = program.arena.stmt(func.body[1]) else {
            panic!("expected plain assignment statement");
        };
        assert_eq!(program.arena.symbol_name(*name), "score");
        assert!(matches!(
            program.arena.expr(*value),
            Expr::NumericLiteral(NumericLiteral::I32(1))
        ));
    }

    #[test]
    fn rustlike_parser_accepts_const_declaration() {
        let src = r#"
fn main() {
    const total: f64 = 1.0 + 2.0;
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("const declaration should parse");
        let func = &program.functions[0];
        let Stmt::Const { name, ty, value } = program.arena.stmt(func.body[0]) else {
            panic!("expected const statement");
        };
        assert_eq!(program.arena.symbol_name(*name), "total");
        assert_eq!(*ty, Some(Type::F64));
        assert!(matches!(
            program.arena.expr(*value),
            Expr::Binary(_, BinaryOp::Add, _)
        ));
    }

    #[test]
    fn rustlike_parser_accepts_extended_numeric_literals() {
        let src = r#"
fn main() {
    let a: i32 = 0xff;
    let b: u32 = 1_000u32;
    let c: f64 = 1_000.25f64;
    let d: fx = 1.25fx;
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("extended numeric literals should parse");
        let func = &program.functions[0];

        let Stmt::Let { value: a, .. } = program.arena.stmt(func.body[0]) else {
            panic!("expected first let statement");
        };
        assert!(matches!(
            program.arena.expr(*a),
            Expr::NumericLiteral(NumericLiteral::I32(255))
        ));

        let Stmt::Let { value: b, .. } = program.arena.stmt(func.body[1]) else {
            panic!("expected second let statement");
        };
        assert!(matches!(
            program.arena.expr(*b),
            Expr::NumericLiteral(NumericLiteral::U32(1000))
        ));

        let Stmt::Let { value: c, .. } = program.arena.stmt(func.body[2]) else {
            panic!("expected third let statement");
        };
        assert!(matches!(
            program.arena.expr(*c),
            Expr::NumericLiteral(NumericLiteral::F64(v)) if (*v - 1000.25).abs() < f64::EPSILON
        ));

        let Stmt::Let { value: d, .. } = program.arena.stmt(func.body[3]) else {
            panic!("expected fourth let statement");
        };
        assert!(matches!(
            program.arena.expr(*d),
            Expr::NumericLiteral(NumericLiteral::Fx(v)) if (*v - 1.25).abs() < f64::EPSILON
        ));
    }

    #[test]
    fn rustlike_parser_accepts_text_literal_and_text_type_surface() {
        let src = r#"
fn main() {
    let message: text = "hello";
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("text literal and text type should parse");
        let func = &program.functions[0];

        let Stmt::Let {
            name,
            ty: Some(Type::Text),
            value,
            ..
        } = program.arena.stmt(func.body[0])
        else {
            panic!("expected text-typed let binding");
        };

        assert_eq!(program.arena.symbol_name(*name), "message");
        assert!(matches!(
            program.arena.expr(*value),
            Expr::TextLiteral(TextLiteral {
                family: TextLiteralFamily::DoubleQuotedUtf8,
                spelling,
            }) if spelling == "\"hello\""
        ));
    }

    #[test]
    fn rustlike_parser_accepts_sequence_literal_and_sequence_type_surface() {
        let src = r#"
fn main() {
    let values: Sequence(i32) = [1, 2, 3];
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("sequence literal and Sequence(type) should parse");
        let func = &program.functions[0];

        let Stmt::Let {
            name,
            ty: Some(Type::Sequence(sequence_ty)),
            value,
            ..
        } = program.arena.stmt(func.body[0])
        else {
            panic!("expected sequence-typed let binding");
        };

        assert_eq!(program.arena.symbol_name(*name), "values");
        assert_eq!(
            sequence_ty.family,
            SequenceCollectionFamily::OrderedSequence
        );
        assert_eq!(sequence_ty.item.as_ref(), &Type::I32);
        assert!(matches!(
            program.arena.expr(*value),
            Expr::SequenceLiteral(SequenceLiteral {
                family: SequenceCollectionFamily::OrderedSequence,
                items,
            }) if items.len() == 3
        ));
    }

    #[test]
    fn rustlike_parser_accepts_sequence_index_surface() {
        let src = r#"
fn main() {
    let values: Sequence(i32) = [1, 2, 3];
    let first: i32 = values[0];
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("sequence index surface should parse");
        let func = &program.functions[0];

        let Stmt::Let {
            ty: Some(Type::I32),
            value,
            ..
        } = program.arena.stmt(func.body[1])
        else {
            panic!("expected indexed let binding");
        };

        let Expr::SequenceIndex(index_expr) = program.arena.expr(*value) else {
            panic!("expected sequence index expression");
        };
        assert!(matches!(program.arena.expr(index_expr.base), Expr::Var(_)));
        assert!(matches!(
            program.arena.expr(index_expr.index),
            Expr::NumericLiteral(NumericLiteral::I32(0))
        ));
    }

    #[test]
    fn strict_profile_accepts_explicit_fx_literals_without_f64_surface() {
        let src = r#"
fn main() {
    let value: fx = 1.25fx;
    return;
}
"#;

        let mut profile = ParserProfile::foundation_default();
        profile.features.allow_f64_math = false;
        parse_rustlike_with_profile(src, &profile)
            .expect("explicit fx literal should not require f64 surface");
    }

    #[test]
    fn rustlike_parser_rejects_hex_f64_literal_surface() {
        let src = r#"
fn main() {
    let value: f64 = 0xfff64;
    return;
}
"#;

        let err = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect_err("hex f64 literal must reject");
        assert!(err
            .message
            .contains("f64 literal currently requires decimal form"));
    }

    #[test]
    fn rustlike_parser_accepts_pipeline_chain() {
        let src = r#"
fn inc(x: f64) -> f64 = x + 1.0;
fn scale(x: f64, factor: f64) -> f64 = x * factor;
fn main() {
    let value: f64 = 1.0 |> inc() |> scale(3.0);
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("pipeline chain should parse");
        let func = &program.functions[2];
        let Stmt::Let { value, .. } = program.arena.stmt(func.body[0]) else {
            panic!("expected leading let statement");
        };
        let Expr::Call(scale_name, scale_args) = program.arena.expr(*value) else {
            panic!("expected outer desugared call");
        };
        assert_eq!(program.arena.symbol_name(*scale_name), "scale");
        assert_eq!(scale_args.len(), 2);
        assert!(scale_args[0].name.is_none());
        let Expr::Call(inc_name, inc_args) = program.arena.expr(scale_args[0].value) else {
            panic!("expected nested pipeline call");
        };
        assert_eq!(program.arena.symbol_name(*inc_name), "inc");
        assert_eq!(inc_args.len(), 1);
        assert!(inc_args[0].name.is_none());
    }

    #[test]
    fn rustlike_parser_accepts_named_arguments() {
        let src = r#"
fn scale(x: f64, factor: f64) -> f64 = x * factor;
fn main() {
    let value: f64 = scale(factor = 3.0, x = 1.0);
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("named arguments should parse");
        let func = &program.functions[1];
        let Stmt::Let { value, .. } = program.arena.stmt(func.body[0]) else {
            panic!("expected leading let statement");
        };
        let Expr::Call(scale_name, scale_args) = program.arena.expr(*value) else {
            panic!("expected call expression");
        };
        assert_eq!(program.arena.symbol_name(*scale_name), "scale");
        assert_eq!(scale_args.len(), 2);
        assert_eq!(
            scale_args[0]
                .name
                .map(|name| program.arena.symbol_name(name)),
            Some("factor")
        );
        assert_eq!(
            scale_args[1]
                .name
                .map(|name| program.arena.symbol_name(name)),
            Some("x")
        );
    }

    #[test]
    fn rustlike_parser_accepts_default_parameters() {
        let src = r#"
fn scale(x: f64, factor: f64 = 2.0) -> f64 = x * factor;
fn main() {
    let value: f64 = scale(3.0);
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("default parameters should parse");
        let scale = &program.functions[0];
        assert_eq!(scale.params.len(), 2);
        assert_eq!(scale.param_defaults.len(), 2);
        assert!(scale.param_defaults[0].is_none());
        let default_expr = scale.param_defaults[1].expect("expected trailing default");
        assert!(matches!(
            program.arena.expr(default_expr),
            Expr::NumericLiteral(NumericLiteral::F64(v)) if (*v - 2.0).abs() < f64::EPSILON
        ));
    }

    #[test]
    fn rustlike_parser_accepts_pipeline_named_arguments_after_prefix() {
        let src = r#"
fn scale(x: f64, factor: f64) -> f64 = x * factor;
fn main() {
    let value: f64 = 1.0 |> scale(factor = 3.0);
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("pipeline named arguments should parse");
        let func = &program.functions[1];
        let Stmt::Let { value, .. } = program.arena.stmt(func.body[0]) else {
            panic!("expected leading let statement");
        };
        let Expr::Call(_, scale_args) = program.arena.expr(*value) else {
            panic!("expected call expression");
        };
        assert_eq!(scale_args.len(), 2);
        assert!(scale_args[0].name.is_none());
        assert_eq!(
            scale_args[1]
                .name
                .map(|name| program.arena.symbol_name(name)),
            Some("factor")
        );
    }

    #[test]
    fn rustlike_parser_accepts_immediate_short_lambda() {
        let src = r#"
fn main() {
    let value: f64 = (x => x + 1.0)(2.0);
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("immediate short lambda should parse");
        let func = &program.functions[0];
        let Stmt::Let { value, .. } = program.arena.stmt(func.body[0]) else {
            panic!("expected leading let statement");
        };
        let Expr::Block(block) = program.arena.expr(*value) else {
            panic!("expected desugared block expression");
        };
        assert_eq!(block.statements.len(), 1);
        let Stmt::Let { name, value, .. } = program.arena.stmt(block.statements[0]) else {
            panic!("expected lambda parameter binding");
        };
        assert_eq!(program.arena.symbol_name(*name), "x");
        assert!(matches!(
            program.arena.expr(*value),
            Expr::NumericLiteral(NumericLiteral::F64(_))
        ));
        assert!(matches!(
            program.arena.expr(block.tail),
            Expr::Binary(_, BinaryOp::Add, _)
        ));
    }

    #[test]
    fn rustlike_parser_accepts_pipeline_short_lambda_stage() {
        let src = r#"
fn main() {
    let value: f64 = 2.0 |> (x => x + 1.0);
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("pipeline short lambda should parse");
        let func = &program.functions[0];
        let Stmt::Let { value, .. } = program.arena.stmt(func.body[0]) else {
            panic!("expected leading let statement");
        };
        let Expr::Block(block) = program.arena.expr(*value) else {
            panic!("expected desugared block expression");
        };
        let Stmt::Let { name, .. } = program.arena.stmt(block.statements[0]) else {
            panic!("expected lambda parameter binding");
        };
        assert_eq!(program.arena.symbol_name(*name), "x");
    }

    #[test]
    fn rustlike_parser_accepts_standalone_first_class_closure_value() {
        let src = r#"
fn main() {
    let value: Closure(f64 -> f64) = (x => x + 1.0);
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("standalone first-class closure should parse");
        let func = &program.functions[0];
        let Stmt::Let {
            ty: Some(Type::Closure(closure_ty)),
            value,
            ..
        } = program.arena.stmt(func.body[0])
        else {
            panic!("expected typed let closure statement");
        };
        assert_eq!(closure_ty.family, ClosureValueFamily::UnaryDirect);
        assert_eq!(closure_ty.capture, ClosureCapturePolicy::Immutable);
        let Expr::Closure(closure) = program.arena.expr(*value) else {
            panic!("expected closure literal");
        };
        assert_eq!(closure.family, ClosureValueFamily::UnaryDirect);
        assert_eq!(closure.capture, ClosureCapturePolicy::Immutable);
        assert!(closure.captures.is_empty());
    }

    #[test]
    fn rustlike_parser_collects_first_class_closure_capture_inventory() {
        let src = r#"
fn main() {
    let offset: f64 = 1.0;
    let value: Closure(f64 -> f64) = (x => x + offset);
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("capturing closure literal should parse");
        let func = &program.functions[0];
        let Stmt::Let { value, .. } = program.arena.stmt(func.body[1]) else {
            panic!("expected closure let statement");
        };
        let Expr::Closure(closure) = program.arena.expr(*value) else {
            panic!("expected closure literal");
        };
        assert_eq!(closure.captures.len(), 1);
        assert_eq!(program.arena.symbol_name(closure.captures[0]), "offset");
    }

    #[test]
    fn rustlike_parser_accepts_closure_type_in_function_signature() {
        let src = r#"
fn id(value: Closure(f64 -> f64)) -> Closure(f64 -> f64) = value;
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("closure type in signature should parse");
        let func = &program.functions[0];
        assert_eq!(
            func.params[0].1,
            Type::Closure(crate::types::ClosureType {
                family: ClosureValueFamily::UnaryDirect,
                capture: ClosureCapturePolicy::Immutable,
                param: Box::new(Type::F64),
                ret: Box::new(Type::F64),
            })
        );
        assert_eq!(func.ret, func.params[0].1);
    }

    #[test]
    fn rustlike_parser_rejects_captureful_short_lambda() {
        let src = r#"
fn main() {
    let offset: f64 = 1.0;
    let value: f64 = (x => x + offset)(2.0);
    return;
}
"#;

        let err = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect_err("captureful short lambda must reject");
        assert!(err.message.contains("capture-free only"));
    }

    #[test]
    fn rustlike_parser_rejects_pipeline_without_call_target() {
        let src = r#"
fn main() {
    let value: f64 = 1.0 |> true;
    return;
}
"#;

        let err = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect_err("pipeline stage without function target must reject");
        assert!(err
            .message
            .contains("pipeline stage must start with function name or call"));
    }

    #[test]
    fn rustlike_parser_rejects_positional_after_named_arguments() {
        let src = r#"
fn scale(x: f64, factor: f64) -> f64 = x * factor;
fn main() {
    let value: f64 = scale(x = 1.0, 3.0);
    return;
}
"#;

        let err = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect_err("positional after named must reject");
        assert!(err
            .message
            .contains("positional arguments cannot follow named arguments"));
    }

    #[test]
    fn rustlike_parser_rejects_required_parameter_after_default() {
        let src = r#"
fn scale(x: f64 = 2.0, factor: f64) -> f64 = x * factor;
fn main() {
    return;
}
"#;

        let err = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect_err("required parameter after default must reject");
        assert!(err
            .message
            .contains("required parameter cannot follow parameter with default value"));
    }

    #[test]
    fn rustlike_parser_accepts_block_expression_tail() {
        let src = r#"
fn main() {
    let value: f64 = {
        const offset: f64 = 2.0;
        let base: f64 = 1.0;
        base + offset
    };
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("block expression should parse");
        let func = &program.functions[0];
        let Stmt::Let { value, .. } = program.arena.stmt(func.body[0]) else {
            panic!("expected leading let statement");
        };
        let Expr::Block(block) = program.arena.expr(*value) else {
            panic!("expected block expression");
        };
        assert_eq!(block.statements.len(), 2);
        match program.arena.expr(block.tail) {
            Expr::Binary(_, BinaryOp::Add, _) => {}
            other => panic!("expected additive tail expression, got {:?}", other),
        }
    }

    #[test]
    fn rustlike_parser_accepts_where_clause_expression() {
        let src = r#"
fn length_sq(x: f64, y: f64) -> f64 = a + b where a = x * x, b = y * y;

fn main() {
    let value: f64 = length_sq(3.0, 4.0);
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("where-clause should parse");
        let func = &program.functions[0];
        let Stmt::Return(Some(value)) = program.arena.stmt(func.body[0]) else {
            panic!("expected expression-bodied return");
        };
        let Expr::Block(block) = program.arena.expr(*value) else {
            panic!("expected where-clause to desugar to block expression");
        };
        assert_eq!(block.statements.len(), 2);
        let Stmt::Let { name, .. } = program.arena.stmt(block.statements[0]) else {
            panic!("expected first where binding");
        };
        assert_eq!(program.arena.symbol_name(*name), "a");
        let Stmt::Let { name, .. } = program.arena.stmt(block.statements[1]) else {
            panic!("expected second where binding");
        };
        assert_eq!(program.arena.symbol_name(*name), "b");
    }

    #[test]
    fn rustlike_parser_accepts_typed_where_binding() {
        let src = r#"
fn main() {
    let value: f64 = total where total: f64 = 1.0;
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("typed where binding should parse");
        let func = &program.functions[0];
        let Stmt::Let { value, .. } = program.arena.stmt(func.body[0]) else {
            panic!("expected leading let statement");
        };
        let Expr::Block(block) = program.arena.expr(*value) else {
            panic!("expected desugared block expression");
        };
        let Stmt::Let { ty, .. } = program.arena.stmt(block.statements[0]) else {
            panic!("expected typed where binding");
        };
        assert_eq!(*ty, Some(Type::F64));
    }

    #[test]
    fn rustlike_parser_rejects_block_expression_without_tail() {
        let src = r#"
fn main() {
    let value: i32 = {
        let base: i32 = 1;
    };
    return;
}
"#;

        let err = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect_err("block expression without tail must reject");
        assert!(err
            .message
            .contains("value-producing block requires trailing value expression"));
    }

    #[test]
    fn rustlike_parser_rejects_where_without_binding() {
        let src = r#"
fn main() {
    let value: f64 = 1.0 where;
    return;
}
"#;

        let err = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect_err("where without binding must reject");
        assert!(err.message.contains("expected identifier"));
    }

    #[test]
    fn rustlike_parser_accepts_if_expression() {
        let src = r#"
fn main() {
    let value: f64 = if true { 1.0 } else { 2.0 };
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("if expression should parse");
        let func = &program.functions[0];
        let Stmt::Let { value, .. } = program.arena.stmt(func.body[0]) else {
            panic!("expected leading let statement");
        };
        let Expr::If(if_expr) = program.arena.expr(*value) else {
            panic!("expected if expression");
        };
        assert!(matches!(
            program.arena.expr(if_expr.condition),
            Expr::BoolLiteral(true)
        ));
    }

    #[test]
    fn rustlike_parser_rejects_if_expression_without_else() {
        let src = r#"
fn main() {
    let value: f64 = if true { 1.0 };
    return;
}
"#;

        let err = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect_err("if expression without else must reject");
        assert!(err
            .message
            .contains("if expression requires explicit else branch"));
    }

    #[test]
    fn rustlike_parser_accepts_else_if_sugar_in_if_expression() {
        let src = r#"
fn main() {
    let value: f64 = if true { 1.0 } else if false { 2.0 } else { 3.0 };
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("else-if sugar should parse in value position");
        let func = &program.functions[0];
        let Stmt::Let { value, .. } = program.arena.stmt(func.body[0]) else {
            panic!("expected leading let statement");
        };
        let Expr::If(outer_if) = program.arena.expr(*value) else {
            panic!("expected if expression");
        };
        let Expr::If(inner_if) = program.arena.expr(outer_if.else_block.tail) else {
            panic!("expected nested if expression in else-if sugar");
        };
        let Expr::BoolLiteral(true) = program.arena.expr(outer_if.condition) else {
            panic!("expected outer condition");
        };
        let Expr::BoolLiteral(false) = program.arena.expr(inner_if.condition) else {
            panic!("expected nested condition");
        };
    }

    #[test]
    fn rustlike_parser_accepts_is_predicate_as_equality_sugar() {
        let src = r#"
fn main() {
    let q: quad = T;
    let value: bool = q is S;
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("is predicate should parse as equality sugar");
        let func = &program.functions[0];
        let Stmt::Let { value, .. } = program.arena.stmt(func.body[1]) else {
            panic!("expected predicate let statement");
        };
        let Expr::Binary(_, BinaryOp::Eq, _) = program.arena.expr(*value) else {
            panic!("expected is predicate to lower to equality in the AST");
        };
    }

    #[test]
    fn rustlike_parser_accepts_when_surface_syntax() {
        let src = r#"
fn main() {
    let value: f64 = when true { 1.0 } else { 2.0 };
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("when expression should parse");
        let func = &program.functions[0];
        let Stmt::Let { value, .. } = program.arena.stmt(func.body[0]) else {
            panic!("expected leading let statement");
        };
        let Expr::If(_) = program.arena.expr(*value) else {
            panic!("expected when surface syntax to lower to if expression");
        };
    }

    #[test]
    fn rustlike_parser_accepts_match_expression() {
        let src = r#"
fn main() {
    let value: f64 = match T {
        T if true => { 1.0 }
        _ => { 2.0 }
    };
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("match expression should parse");
        let func = &program.functions[0];
        let Stmt::Let { value, .. } = program.arena.stmt(func.body[0]) else {
            panic!("expected leading let statement");
        };
        let Expr::Match(match_expr) = program.arena.expr(*value) else {
            panic!("expected match expression");
        };
        assert!(matches!(
            program.arena.expr(match_expr.scrutinee),
            Expr::QuadLiteral(QuadVal::T)
        ));
        assert_eq!(match_expr.arms.len(), 1);
        assert!(match_expr.arms[0].guard.is_some());
        assert!(match_expr.default.is_some());
    }

    #[test]
    fn rustlike_parser_accepts_adt_match_expression_surface() {
        let src = r#"
enum Maybe {
    None,
    Some(f64),
}

fn main() {
    let value: f64 = match Maybe::Some(1.0) {
        Maybe::Some(total) => { total }
        _ => { 0.0 }
    };
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("ADT match expression should parse");
        let func = &program.functions[0];
        let Stmt::Let { value, .. } = program.arena.stmt(func.body[0]) else {
            panic!("expected leading let statement");
        };
        let Expr::Match(match_expr) = program.arena.expr(*value) else {
            panic!("expected match expression");
        };
        assert_eq!(match_expr.arms.len(), 1);
        let MatchPattern::Adt(pat) = &match_expr.arms[0].pat else {
            panic!("expected ADT match pattern");
        };
        assert_eq!(program.arena.symbol_name(pat.adt_name), "Maybe");
        assert_eq!(program.arena.symbol_name(pat.variant_name), "Some");
        assert!(matches!(
            pat.items.as_slice(),
            [AdtPatternItem::Bind { .. }]
        ));
        assert!(match_expr.default.is_some());
    }

    #[test]
    fn rustlike_parser_rejects_guarded_default_match_arm() {
        let src = r#"
fn main() {
    let value: f64 = match T {
        _ if true => { 2.0 }
    };
    return;
}
"#;

        let err = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect_err("guarded default arm must reject");
        assert!(err
            .message
            .contains("default '_' arm in match currently cannot have guard"));
    }

    #[test]
    fn rustlike_parser_accepts_guard_clause() {
        let src = r#"
fn main() {
    guard true else return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("guard clause should parse");
        let func = &program.functions[0];
        let Stmt::Guard {
            condition,
            else_return,
        } = program.arena.stmt(func.body[0])
        else {
            panic!("expected guard statement");
        };
        assert!(matches!(
            program.arena.expr(*condition),
            Expr::BoolLiteral(true)
        ));
        assert!(else_return.is_none());
    }

    #[test]
    fn rustlike_parser_accepts_for_over_range_statement() {
        let src = r#"
fn main() {
    for i in 0..=2 {
        let _ = i;
    }
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("for-range statement should parse");
        let func = &program.functions[0];
        let Stmt::ForRange { name, range, body } = program.arena.stmt(func.body[0]) else {
            panic!("expected for-range statement");
        };
        assert_eq!(program.arena.symbol_name(*name), "i");
        let Expr::Range(range_expr) = program.arena.expr(*range) else {
            panic!("expected range expression");
        };
        assert!(range_expr.inclusive);
        assert_eq!(body.len(), 1);
    }

    #[test]
    fn rustlike_parser_owns_iterable_for_surface_separately_from_range() {
        let src = r#"
fn main() {
    let items: Sequence(i32) = [1, 2, 3];
    for item in items {
        let _ = item;
    }
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("iterable owner-layer for-loop should parse");
        let func = &program.functions[0];
        let Stmt::ForEach {
            name,
            iterable,
            body,
            desugaring,
        } = program.arena.stmt(func.body[1])
        else {
            panic!("expected owner-layer iterable for-loop");
        };
        assert_eq!(program.arena.symbol_name(*name), "item");
        assert!(matches!(program.arena.expr(*iterable), Expr::Var(_)));
        assert_eq!(program.arena.symbol_name(desugaring.trait_name), "Iterable");
        assert_eq!(body.len(), 1);
    }

    #[test]
    fn rustlike_parser_rejects_guard_without_else_return() {
        let src = r#"
fn main() {
    guard true else { return; }
}
"#;

        let err = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect_err("guard clause without else return should reject");
        assert!(err
            .message
            .contains("guard clause currently supports only else return"));
    }

    #[test]
    fn rustlike_parser_accepts_discard_bind() {
        let src = r#"
fn main() {
    let _ = 1.0;
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("discard bind should parse");
        let func = &program.functions[0];
        let Stmt::Discard { ty, value } = program.arena.stmt(func.body[0]) else {
            panic!("expected discard statement");
        };
        assert!(ty.is_none());
        assert!(matches!(
            program.arena.expr(*value),
            Expr::NumericLiteral(NumericLiteral::F64(_))
        ));
    }

    #[test]
    fn rustlike_parser_accepts_typed_discard_bind() {
        let src = r#"
fn main() {
    let _: f64 = 1.0;
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("typed discard bind should parse");
        let func = &program.functions[0];
        let Stmt::Discard { ty, value } = program.arena.stmt(func.body[0]) else {
            panic!("expected discard statement");
        };
        assert_eq!(*ty, Some(Type::F64));
        assert!(matches!(
            program.arena.expr(*value),
            Expr::NumericLiteral(NumericLiteral::F64(_))
        ));
    }

    #[test]
    fn rustlike_parser_accepts_tuple_destructuring_bind() {
        let src = r#"
fn pair() -> (i32, bool) = (1, true);

fn main() {
    let (count, _): (i32, bool) = pair();
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("tuple destructuring bind should parse");
        let func = &program.functions[1];
        let Stmt::LetTuple { items, ty, value } = program.arena.stmt(func.body[0]) else {
            panic!("expected tuple destructuring statement");
        };
        assert_eq!(items.len(), 2);
        assert!(matches!(
            items[0],
            TuplePatternItem::Bind {
                name,
                capture: CaptureMode::Move
            } if program.arena.symbol_name(name) == "count"
        ));
        assert!(matches!(items[1], TuplePatternItem::Discard));
        assert_eq!(*ty, Some(Type::Tuple(vec![Type::I32, Type::Bool])));
        let Expr::Call(name, args) = program.arena.expr(*value) else {
            panic!("expected call expression");
        };
        assert_eq!(program.arena.symbol_name(*name), "pair");
        assert!(args.is_empty());
    }

    #[test]
    fn rustlike_parser_preserves_borrow_capture_in_plain_tuple_bind() {
        let src = r#"
fn pair() -> (i32, bool) = (1, true);

fn main() {
    let (ref count, ready) = pair();
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("tuple ref bind should parse");
        let func = &program.functions[1];
        let Stmt::LetTuple { items, .. } = program.arena.stmt(func.body[0]) else {
            panic!("expected tuple destructuring statement");
        };
        assert!(matches!(
            items[0],
            TuplePatternItem::Bind {
                name,
                capture: CaptureMode::Borrow
            } if program.arena.symbol_name(name) == "count"
        ));
        assert!(matches!(
            items[1],
            TuplePatternItem::Bind {
                name,
                capture: CaptureMode::Move
            } if program.arena.symbol_name(name) == "ready"
        ));
    }

    #[test]
    fn rustlike_parser_accepts_tuple_let_else_surface() {
        let src = r#"
fn pair() -> (i32, quad) = (1, T);

fn main() {
    let (count, T): (i32, quad) = pair() else return;
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("tuple let-else should parse");
        let func = &program.functions[1];
        let Stmt::LetElseTuple {
            items,
            ty,
            value,
            else_return,
        } = program.arena.stmt(func.body[0])
        else {
            panic!("expected tuple let-else statement");
        };
        assert_eq!(items.len(), 2);
        assert!(matches!(items[0], TuplePatternItem::Bind { .. }));
        assert!(matches!(
            items[1],
            TuplePatternItem::QuadLiteral(QuadVal::T)
        ));
        assert_eq!(*ty, Some(Type::Tuple(vec![Type::I32, Type::Quad])));
        assert!(else_return.is_none());
        let Expr::Call(name, args) = program.arena.expr(*value) else {
            panic!("expected call expression");
        };
        assert_eq!(program.arena.symbol_name(*name), "pair");
        assert!(args.is_empty());
    }

    #[test]
    fn rustlike_parser_rejects_plain_bind_let_else_surface() {
        let src = r#"
fn main() {
    let value = 1 else return;
    return;
}
"#;

        let err = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect_err("plain let-else target must reject");
        assert!(err
            .message
            .contains("let-else currently requires tuple destructuring target"));
    }

    #[test]
    fn rustlike_parser_rejects_non_return_tuple_let_else_surface() {
        let src = r#"
fn pair() -> (i32, quad) = (1, T);

fn main() {
    let (count, T) = pair() else guard true else return;
    return;
}
"#;

        let err = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect_err("tuple let-else non-return branch must reject");
        assert!(err
            .message
            .contains("let-else currently supports only else return"));
    }

    #[test]
    fn rustlike_parser_accepts_tuple_destructuring_assignment() {
        let src = r#"
fn pair() -> (i32, bool) = (1, true);

fn main() {
    let count: i32 = 0;
    let ready: bool = false;
    (count, ready) = pair();
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("tuple destructuring assignment should parse");
        let func = &program.functions[1];
        let Stmt::AssignTuple { items, value } = program.arena.stmt(func.body[2]) else {
            panic!("expected tuple destructuring assignment");
        };
        assert_eq!(items.len(), 2);
        assert_eq!(
            items[0].map(|name| program.arena.symbol_name(name)),
            Some("count")
        );
        assert_eq!(
            items[1].map(|name| program.arena.symbol_name(name)),
            Some("ready")
        );
        let Expr::Call(name, args) = program.arena.expr(*value) else {
            panic!("expected call expression");
        };
        assert_eq!(program.arena.symbol_name(*name), "pair");
        assert!(args.is_empty());
    }

    #[test]
    fn rustlike_parser_accepts_top_level_record_declaration() {
        let src = r#"
record DecisionContext {
    camera: quad,
    badge: quad,
    quality: f64,
}

fn main() {
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("record declaration should parse");
        assert_eq!(program.records.len(), 1);
        let record = &program.records[0];
        assert_eq!(program.arena.symbol_name(record.name), "DecisionContext");
        assert_eq!(record.fields.len(), 3);
        assert_eq!(program.arena.symbol_name(record.fields[0].name), "camera");
        assert_eq!(record.fields[0].ty, Type::Quad);
        assert_eq!(record.fields[2].ty, Type::F64);
        assert_eq!(program.functions.len(), 1);
    }

    #[test]
    fn rustlike_parser_accepts_top_level_schema_declaration() {
        let src = r#"
schema SensorConfig {
    interval_ms: u32[ms],
    fallback: Option(quad),
}

fn main() {
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("schema declaration should parse");
        assert_eq!(program.schemas.len(), 1);
        let schema = &program.schemas[0];
        assert_eq!(program.arena.symbol_name(schema.name), "SensorConfig");
        let SchemaShape::Record(fields) = &schema.shape else {
            panic!("expected record-shaped schema");
        };
        assert_eq!(fields.len(), 2);
        assert_eq!(program.arena.symbol_name(fields[0].name), "interval_ms");
    }

    #[test]
    fn rustlike_parser_accepts_tagged_union_schema_declaration() {
        let src = r#"
schema Message {
    Ping {},
    Data {
        value: f64,
        status: Result(quad, bool),
    },
}

fn main() {
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("tagged-union schema declaration should parse");
        assert_eq!(program.schemas.len(), 1);
        let schema = &program.schemas[0];
        assert_eq!(program.arena.symbol_name(schema.name), "Message");
        let SchemaShape::TaggedUnion(variants) = &schema.shape else {
            panic!("expected tagged-union schema");
        };
        assert_eq!(variants.len(), 2);
        assert_eq!(program.arena.symbol_name(variants[0].name), "Ping");
        assert!(variants[0].fields.is_empty());
        assert_eq!(program.arena.symbol_name(variants[1].name), "Data");
        assert_eq!(variants[1].fields.len(), 2);
    }

    #[test]
    fn rustlike_parser_accepts_role_marked_schema_declarations() {
        let src = r#"
config schema AppConfig {
    interval_ms: u32[ms],
}

wire schema Envelope {
    Ping {},
    Data {
        value: f64,
    },
}

fn main() {
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("role-marked schema declarations should parse");
        assert_eq!(program.schemas.len(), 2);
        assert_eq!(program.schemas[0].role, Some(SchemaRole::Config));
        assert_eq!(program.schemas[1].role, Some(SchemaRole::Wire));
    }

    #[test]
    fn rustlike_parser_accepts_schema_version_marker() {
        let src = r#"
api schema Envelope version(2) {
    Data {
        sample_count: i32,
    },
}

fn main() {
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("schema version marker should parse");
        assert_eq!(program.schemas.len(), 1);
        assert_eq!(program.schemas[0].role, Some(SchemaRole::Api));
        assert_eq!(program.schemas[0].version, Some(SchemaVersion { value: 2 }));
    }

    #[test]
    fn rustlike_parser_rejects_zero_schema_version_marker() {
        let src = r#"
schema Envelope version(0) {
    value: i32,
}

fn main() {
    return;
}
"#;

        let err = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect_err("zero schema version must reject");
        assert!(err
            .message
            .contains("schema version marker must be positive"));
    }

    #[test]
    fn rustlike_parser_rejects_suffixed_schema_version_marker() {
        let src = r#"
schema Envelope version(1u32) {
    value: i32,
}

fn main() {
    return;
}
"#;

        let err = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect_err("suffixed schema version must reject");
        assert!(err
            .message
            .contains("schema version marker currently requires unsuffixed decimal integer"));
    }

    #[test]
    fn strict_profile_rejects_schema_surface() {
        let src = r#"
schema SensorConfig {
    interval_ms: u32,
}

fn main() {
    return;
}
"#;

        let err = parse_rustlike_with_profile(src, &ParserProfile::default())
            .expect_err("strict profile must reject schema surface");

        assert_eq!(err.kind(), FrontendErrorKind::PolicyViolation);
        assert!(err.message.contains("schema"));
    }

    #[test]
    fn rustlike_parser_accepts_top_level_enum_and_constructor_surface() {
        let src = r#"
enum Maybe {
    None,
    Some(bool),
}

fn main() {
    let left: Maybe = Maybe::Some(true);
    let right: Maybe = Maybe::None;
    let _ = left;
    let _ = right;
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("enum declaration should parse");
        assert_eq!(program.adts.len(), 1);
        let adt = &program.adts[0];
        assert_eq!(program.arena.symbol_name(adt.name), "Maybe");
        assert_eq!(adt.variants.len(), 2);
        assert_eq!(program.arena.symbol_name(adt.variants[0].name), "None");
        assert!(adt.variants[0].payload.is_empty());
        assert_eq!(program.arena.symbol_name(adt.variants[1].name), "Some");
        assert_eq!(adt.variants[1].payload, vec![Type::Bool]);

        let main = &program.functions[0];
        match program.arena.stmt(main.body[0]) {
            Stmt::Let { value, .. } => match program.arena.expr(*value) {
                Expr::AdtCtor(ctor) => {
                    assert_eq!(program.arena.symbol_name(ctor.adt_name), "Maybe");
                    assert_eq!(program.arena.symbol_name(ctor.variant_name), "Some");
                    assert_eq!(ctor.payload.len(), 1);
                }
                other => panic!("expected adt constructor expression, got {:?}", other),
            },
            other => panic!("expected let binding, got {:?}", other),
        }
        match program.arena.stmt(main.body[1]) {
            Stmt::Let { value, .. } => match program.arena.expr(*value) {
                Expr::AdtCtor(ctor) => {
                    assert_eq!(program.arena.symbol_name(ctor.variant_name), "None");
                    assert!(ctor.payload.is_empty());
                }
                other => panic!("expected adt constructor expression, got {:?}", other),
            },
            other => panic!("expected let binding, got {:?}", other),
        }
    }

    #[test]
    fn rustlike_parser_accepts_option_and_result_standard_form_types() {
        let src = r#"
fn wrap(flag: bool) -> Option(bool) {
    let some: Option(bool) = Option::Some(flag);
    let none: Option(bool) = Option::None;
    let ok: Result(bool, quad) = Result::Ok(flag);
    let err: Result(bool, quad) = Result::Err(N);
    let _ = some;
    let _ = none;
    let _ = ok;
    let _ = err;
    return Option::Some(flag);
}

fn main() {
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("Option/Result standard-form surface should parse");
        let wrap = &program.functions[0];
        assert_eq!(wrap.ret, Type::Option(Box::new(Type::Bool)));
        assert_eq!(wrap.params[0].1, Type::Bool);
        match program.arena.stmt(wrap.body[0]) {
            Stmt::Let {
                ty: Some(ty),
                value,
                ..
            } => {
                assert_eq!(*ty, Type::Option(Box::new(Type::Bool)));
                let Expr::AdtCtor(ctor) = program.arena.expr(*value) else {
                    panic!("expected Option constructor expression");
                };
                assert_eq!(program.arena.symbol_name(ctor.adt_name), "Option");
                assert_eq!(program.arena.symbol_name(ctor.variant_name), "Some");
            }
            other => panic!("expected typed let binding, got {:?}", other),
        }
        match program.arena.stmt(wrap.body[2]) {
            Stmt::Let {
                ty: Some(ty),
                value,
                ..
            } => {
                assert_eq!(
                    *ty,
                    Type::Result(Box::new(Type::Bool), Box::new(Type::Quad))
                );
                let Expr::AdtCtor(ctor) = program.arena.expr(*value) else {
                    panic!("expected Result constructor expression");
                };
                assert_eq!(program.arena.symbol_name(ctor.adt_name), "Result");
                assert_eq!(program.arena.symbol_name(ctor.variant_name), "Ok");
            }
            other => panic!("expected typed let binding, got {:?}", other),
        }
    }

    #[test]
    fn rustlike_parser_accepts_while_statement_surface() {
        let src = r#"
fn main() {
    let mut i: i32 = 0;
    while i < 3 {
        i = i + 1;
    }
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("while statement should parse");
        let main = &program.functions[0];
        match program.arena.stmt(main.body[1]) {
            Stmt::While { condition, body } => {
                assert!(matches!(
                    program.arena.expr(*condition),
                    Expr::Binary(_, _, _)
                ));
                assert_eq!(body.len(), 1);
                assert!(matches!(program.arena.stmt(body[0]), Stmt::Assign { .. }));
            }
            other => panic!("expected while statement, got {:?}", other),
        }
    }

    #[test]
    fn rustlike_parser_accepts_option_and_result_match_patterns() {
        let src = r#"
fn unwrap(opt: Option(bool), res: Result(bool, quad)) {
    match opt {
        Option::Some(value) => { let _ = value; }
        _ => { return; }
    }
    match res {
        Result::Ok(value) => { let _ = value; }
        Result::Err(code) => { let _ = code; }
    }
    return;
}

fn main() {
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("Option/Result match patterns should parse");
        let unwrap = &program.functions[0];
        match program.arena.stmt(unwrap.body[0]) {
            Stmt::Match { arms, .. } => {
                let MatchPattern::Adt(pat) = &arms[0].pat else {
                    panic!("expected Option match pattern");
                };
                assert_eq!(program.arena.symbol_name(pat.adt_name), "Option");
                assert_eq!(program.arena.symbol_name(pat.variant_name), "Some");
                assert!(matches!(
                    pat.items.as_slice(),
                    [AdtPatternItem::Bind { .. }]
                ));
            }
            other => panic!("expected match stmt, got {:?}", other),
        }
        match program.arena.stmt(unwrap.body[1]) {
            Stmt::Match { arms, default, .. } => {
                assert!(
                    default.is_empty(),
                    "exhaustive Result match should omit default"
                );
                let MatchPattern::Adt(pat) = &arms[1].pat else {
                    panic!("expected Result match pattern");
                };
                assert_eq!(program.arena.symbol_name(pat.adt_name), "Result");
                assert_eq!(program.arena.symbol_name(pat.variant_name), "Err");
                assert!(matches!(
                    pat.items.as_slice(),
                    [AdtPatternItem::Bind { .. }]
                ));
            }
            other => panic!("expected match stmt, got {:?}", other),
        }
    }

    #[test]
    fn rustlike_parser_accepts_units_of_measure_annotations_in_declared_types() {
        let src = r#"
record Measurement {
    distance: f64[m],
    ticks: u32[ms],
    maybe: Option(f64[m]),
}

fn keep(
    distance: f64[m],
    pair: (f64[m], u32[ms]),
    maybe: Option(f64[m]),
    result: Result(f64[m], quad)
) -> f64[m] {
    let reading: f64[m] = 1.0;
    let _ = pair;
    let _ = maybe;
    let _ = result;
    return reading;
}

fn main() {
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("units-of-measure annotations should parse");
        let record = &program.records[0];
        assert!(matches!(
            &record.fields[0].ty,
            Type::Measured(base, unit)
                if base.as_ref() == &Type::F64 && program.arena.symbol_name(*unit) == "m"
        ));
        assert!(matches!(
            &record.fields[1].ty,
            Type::Measured(base, unit)
                if base.as_ref() == &Type::U32 && program.arena.symbol_name(*unit) == "ms"
        ));
        assert!(matches!(
            &record.fields[2].ty,
            Type::Option(inner)
                if matches!(
                    inner.as_ref(),
                    Type::Measured(base, unit)
                        if base.as_ref() == &Type::F64
                            && program.arena.symbol_name(*unit) == "m"
                )
        ));

        let func = &program.functions[0];
        assert!(matches!(
            &func.params[0].1,
            Type::Measured(base, unit)
                if base.as_ref() == &Type::F64 && program.arena.symbol_name(*unit) == "m"
        ));
        assert!(matches!(
            &func.params[1].1,
            Type::Tuple(items)
                if items.len() == 2
                    && matches!(
                        &items[0],
                        Type::Measured(base, unit)
                            if base.as_ref() == &Type::F64
                                && program.arena.symbol_name(*unit) == "m"
                    )
                    && matches!(
                        &items[1],
                        Type::Measured(base, unit)
                            if base.as_ref() == &Type::U32
                                && program.arena.symbol_name(*unit) == "ms"
                    )
        ));
        assert!(matches!(
            &func.ret,
            Type::Measured(base, unit)
                if base.as_ref() == &Type::F64 && program.arena.symbol_name(*unit) == "m"
        ));
    }

    #[test]
    fn rustlike_parser_rejects_unit_annotation_on_non_numeric_type() {
        let src = r#"
fn main() {
    let bad: bool[m] = true;
    return;
}
"#;

        let err = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect_err("non-numeric unit annotations must reject");
        assert!(err
            .message
            .contains("unit annotation is allowed only on i32, u32, f64, or fx in v0"));
    }

    #[test]
    fn rustlike_parser_accepts_record_type_name_in_function_signature() {
        let src = r#"
record DecisionContext {
    camera: quad,
}

fn describe(ctx: DecisionContext) -> DecisionContext {
    return ctx;
}

fn main() {
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("record type name in signature should parse");
        let func = &program.functions[0];
        assert_eq!(program.arena.symbol_name(func.name), "describe");
        assert!(
            matches!(func.params[0].1, Type::Record(name) if program.arena.symbol_name(name) == "DecisionContext")
        );
        assert!(
            matches!(func.ret, Type::Record(name) if program.arena.symbol_name(name) == "DecisionContext")
        );
    }

    #[test]
    fn rustlike_parser_accepts_empty_record_surface_for_later_sema_rejection() {
        let src = r#"
record Empty {}

fn main() {
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("empty record is rejected in sema, not parser");
        assert_eq!(program.records.len(), 1);
        assert!(program.records[0].fields.is_empty());
    }

    #[test]
    fn rustlike_parser_accepts_record_literal_surface() {
        let src = r#"
record DecisionContext {
    camera: quad,
    quality: f64,
}

fn main() {
    let ctx = DecisionContext { quality: 0.75, camera: T };
    return;
}
        "#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("record literal surface should parse");
        let main = &program.functions[0];
        let Stmt::Let { value, .. } = program.arena.stmt(main.body[0]) else {
            panic!("expected leading let");
        };
        let Expr::RecordLiteral(record) = program.arena.expr(*value) else {
            panic!("expected record literal expr");
        };
        assert_eq!(program.arena.symbol_name(record.name), "DecisionContext");
        assert_eq!(record.fields.len(), 2);
        assert_eq!(program.arena.symbol_name(record.fields[0].name), "quality");
        assert_eq!(program.arena.symbol_name(record.fields[1].name), "camera");
    }

    #[test]
    fn rustlike_parser_accepts_record_field_access_surface() {
        let src = r#"
record DecisionContext {
    camera: quad,
}

fn main() {
    let ctx = DecisionContext { camera: T };
    let camera = ctx.camera;
    return;
}
        "#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("record field access should parse");
        let main = &program.functions[0];
        let Stmt::Let { value, .. } = program.arena.stmt(main.body[1]) else {
            panic!("expected second let");
        };
        let Expr::RecordField(field_expr) = program.arena.expr(*value) else {
            panic!("expected record field expr");
        };
        let Expr::Var(base) = program.arena.expr(field_expr.base) else {
            panic!("expected field access base variable");
        };
        assert_eq!(program.arena.symbol_name(*base), "ctx");
        assert_eq!(program.arena.symbol_name(field_expr.field), "camera");
    }

    #[test]
    fn rustlike_parser_accepts_record_copy_with_surface() {
        let src = r#"
record DecisionContext {
    camera: quad,
    quality: f64,
}

fn main() {
    let ctx = DecisionContext { camera: T, quality: 0.75 };
    let patched = ctx with { quality: 1.0 };
    return;
}
        "#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("record copy-with should parse");
        let main = &program.functions[0];
        let Stmt::Let { value, .. } = program.arena.stmt(main.body[1]) else {
            panic!("expected second let");
        };
        let Expr::RecordUpdate(update_expr) = program.arena.expr(*value) else {
            panic!("expected record update expr");
        };
        let Expr::Var(base) = program.arena.expr(update_expr.base) else {
            panic!("expected record update base variable");
        };
        assert_eq!(program.arena.symbol_name(*base), "ctx");
        assert_eq!(update_expr.fields.len(), 1);
        assert_eq!(
            program.arena.symbol_name(update_expr.fields[0].name),
            "quality"
        );
    }

    #[test]
    fn rustlike_parser_accepts_explicit_record_destructuring_bind() {
        let src = r#"
record DecisionContext {
    camera: quad,
    quality: f64,
}

fn main() {
    let DecisionContext { camera: seen_camera, quality: _ } =
        DecisionContext { camera: T, quality: 0.75 };
    return;
}
        "#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("record destructuring bind should parse");
        let main = &program.functions[0];
        let Stmt::LetRecord {
            record_name,
            items,
            value,
        } = program.arena.stmt(main.body[0])
        else {
            panic!("expected record destructuring let");
        };
        assert_eq!(program.arena.symbol_name(*record_name), "DecisionContext");
        assert_eq!(items.len(), 2);
        assert_eq!(program.arena.symbol_name(items[0].field), "camera");
        assert!(matches!(
            items[0].target,
            RecordPatternTarget::Bind {
                name,
                capture: CaptureMode::Move
            } if program.arena.symbol_name(name) == "seen_camera"
        ));
        assert_eq!(program.arena.symbol_name(items[1].field), "quality");
        assert!(matches!(items[1].target, RecordPatternTarget::Discard));
        assert!(matches!(program.arena.expr(*value), Expr::RecordLiteral(_)));
    }

    #[test]
    fn rustlike_parser_accepts_record_field_shorthand_in_literal_and_copy_with() {
        let src = r#"
record DecisionContext {
    camera: quad,
    quality: f64,
}

fn main() {
    let camera: quad = T;
    let quality: f64 = 0.75;
    let ctx: DecisionContext = DecisionContext { camera, quality };
    let patched: DecisionContext = ctx with { quality };
    return;
}
        "#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("record field shorthand should parse");
        let main = &program.functions[0];
        let Stmt::Let { value, .. } = program.arena.stmt(main.body[2]) else {
            panic!("expected record literal let");
        };
        let Expr::RecordLiteral(record) = program.arena.expr(*value) else {
            panic!("expected record literal expression");
        };
        assert!(
            matches!(program.arena.expr(record.fields[0].value), Expr::Var(name) if program.arena.symbol_name(*name) == "camera")
        );
        assert!(
            matches!(program.arena.expr(record.fields[1].value), Expr::Var(name) if program.arena.symbol_name(*name) == "quality")
        );

        let Stmt::Let { value, .. } = program.arena.stmt(main.body[3]) else {
            panic!("expected record copy-with let");
        };
        let Expr::RecordUpdate(update) = program.arena.expr(*value) else {
            panic!("expected record copy-with expression");
        };
        assert_eq!(update.fields.len(), 1);
        assert!(
            matches!(program.arena.expr(update.fields[0].value), Expr::Var(name) if program.arena.symbol_name(*name) == "quality")
        );
    }

    #[test]
    fn rustlike_parser_accepts_record_pattern_punning_in_bind_and_let_else() {
        let src = r#"
record DecisionContext {
    camera: quad,
    quality: f64,
}

fn main() {
    let DecisionContext { camera, quality: _ } =
        DecisionContext { camera: T, quality: 0.75 };
    let DecisionContext { camera: T, quality } =
        DecisionContext { camera: T, quality: 1.0 } else return;
    return;
}
        "#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("record pattern punning should parse");
        let main = &program.functions[0];
        let Stmt::LetRecord { items, .. } = program.arena.stmt(main.body[0]) else {
            panic!("expected record destructuring bind");
        };
        assert!(matches!(
            items[0].target,
            RecordPatternTarget::Bind {
                name,
                capture: CaptureMode::Move
            } if program.arena.symbol_name(name) == "camera"
        ));
        assert!(matches!(items[1].target, RecordPatternTarget::Discard));

        let Stmt::LetElseRecord { items, .. } = program.arena.stmt(main.body[1]) else {
            panic!("expected record let-else");
        };
        assert!(matches!(
            items[0].target,
            RecordPatternTarget::QuadLiteral(QuadVal::T)
        ));
        assert!(matches!(
            items[1].target,
            RecordPatternTarget::Bind {
                name,
                capture: CaptureMode::Move
            } if program.arena.symbol_name(name) == "quality"
        ));
    }

    #[test]
    fn rustlike_parser_preserves_borrow_capture_in_record_bind() {
        let src = r#"
record DecisionContext {
    camera: quad,
    quality: f64,
}

fn main() {
    let DecisionContext { camera: ref seen_camera, quality } =
        DecisionContext { camera: T, quality: 0.75 };
    return;
}
        "#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("record ref bind should parse");
        let main = &program.functions[0];
        let Stmt::LetRecord { items, .. } = program.arena.stmt(main.body[0]) else {
            panic!("expected record destructuring bind");
        };
        assert!(matches!(
            items[0].target,
            RecordPatternTarget::Bind {
                name,
                capture: CaptureMode::Borrow
            } if program.arena.symbol_name(name) == "seen_camera"
        ));
        assert!(matches!(
            items[1].target,
            RecordPatternTarget::Bind {
                name,
                capture: CaptureMode::Move
            } if program.arena.symbol_name(name) == "quality"
        ));
    }

    #[test]
    fn rustlike_parser_rejects_duplicate_field_in_record_destructuring_bind() {
        let src = r#"
record DecisionContext {
    camera: quad,
}

fn main() {
    let DecisionContext { camera: first, camera: second } =
        DecisionContext { camera: T };
    return;
}
        "#;

        let err = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect_err("duplicate field should reject");
        assert!(err
            .message
            .contains("record destructuring pattern cannot repeat field 'camera'"));
    }

    #[test]
    fn rustlike_parser_rejects_duplicate_binding_in_record_destructuring_bind() {
        let src = r#"
record DecisionContext {
    camera: quad,
    badge: quad,
}

fn main() {
    let DecisionContext { camera: seen, badge: seen } =
        DecisionContext { camera: T, badge: F };
    return;
}
        "#;

        let err = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect_err("duplicate binding should reject");
        assert!(err
            .message
            .contains("record destructuring pattern cannot repeat binding 'seen'"));
    }

    #[test]
    fn rustlike_parser_rejects_duplicate_binding_in_record_destructuring_bind_with_punning() {
        let src = r#"
record DecisionContext {
    camera: quad,
    badge: quad,
}

fn main() {
    let DecisionContext { camera, badge: camera } =
        DecisionContext { camera: T, badge: F };
    return;
}
        "#;

        let err = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect_err("duplicate binding via punning should reject");
        assert!(err
            .message
            .contains("record destructuring pattern cannot repeat binding 'camera'"));
    }

    #[test]
    fn rustlike_parser_accepts_record_let_else_surface() {
        let src = r#"
record DecisionContext {
    camera: quad,
    quality: f64,
}

fn main() {
    let DecisionContext { camera: T, quality: score } =
        DecisionContext { camera: T, quality: 0.75 } else return;
    return;
}
        "#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("record let-else should parse");
        let main = &program.functions[0];
        let Stmt::LetElseRecord {
            record_name,
            items,
            value,
            else_return,
        } = program.arena.stmt(main.body[0])
        else {
            panic!("expected record let-else statement");
        };
        assert_eq!(program.arena.symbol_name(*record_name), "DecisionContext");
        assert_eq!(items.len(), 2);
        assert!(matches!(
            items[0].target,
            RecordPatternTarget::QuadLiteral(QuadVal::T)
        ));
        assert!(matches!(
            items[1].target,
            RecordPatternTarget::Bind {
                name,
                capture: CaptureMode::Move
            } if program.arena.symbol_name(name) == "score"
        ));
        assert!(matches!(program.arena.expr(*value), Expr::RecordLiteral(_)));
        assert!(else_return.is_none());
    }

    #[test]
    fn rustlike_parser_rejects_quad_literal_record_pattern_without_let_else() {
        let src = r#"
record DecisionContext {
    camera: quad,
}

fn main() {
    let DecisionContext { camera: T } = DecisionContext { camera: T };
    return;
}
        "#;

        let err = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect_err("plain record destructuring with quad literal must reject");
        assert!(err
            .message
            .contains("quad literal record field patterns currently require let-else"));
    }

    #[test]
    fn rustlike_parser_admits_nested_tuple_destructuring_bind() {
        // M9.4 Wave 2: nested tuple destructuring is now admitted at parse level.
        // Typecheck support is deferred to Wave 3.
        let src = r#"
fn main() {
    let ((x, y), z) = ((1, true), false);
    return;
}
"#;
        // Parser must accept the shape (typecheck will later reject at Wave 3 stub).
        let _ = parse_rustlike_with_profile(src, &ParserProfile::foundation_default());
    }

    #[test]
    fn rustlike_parser_admits_nested_tuple_destructuring_assignment() {
        // M9.4 Wave 2: nested tuple destructuring admitted at parse level.
        let src = r#"
fn main() {
    let x: i32 = 0;
    let y: bool = false;
    let z: bool = false;
    ((x, y), z) = ((1, true), false);
    return;
}
"#;
        // Parser must accept the shape.
        let _ = parse_rustlike_with_profile(src, &ParserProfile::foundation_default());
    }

    #[test]
    fn rustlike_parser_accepts_assert_statement_surface() {
        let src = r#"
fn main() {
    assert(true);
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("assert statement should parse");
        let func = &program.functions[0];
        let Stmt::Expr(expr_id) = program.arena.stmt(func.body[0]) else {
            panic!("expected expression statement");
        };
        let Expr::Call(name, args) = program.arena.expr(*expr_id) else {
            panic!("expected call-shaped assert surface");
        };
        assert_eq!(program.arena.symbol_name(*name), "assert");
        assert_eq!(args.len(), 1);
        assert!(matches!(
            program.arena.expr(args[0].value),
            Expr::BoolLiteral(true)
        ));
    }

    #[test]
    fn rustlike_parser_accepts_tuple_literal_and_tuple_type_surface() {
        let src = r#"
fn pair() -> (i32, bool) = (1, true);

fn main() {
    let pair: (i32, bool) = (1, true);
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("tuple literal/type surface should parse");
        let pair_fn = &program.functions[0];
        assert_eq!(pair_fn.ret, Type::Tuple(vec![Type::I32, Type::Bool]));

        let main_fn = &program.functions[1];
        let Stmt::Let { ty, value, .. } = program.arena.stmt(main_fn.body[0]) else {
            panic!("expected let statement");
        };
        assert_eq!(ty.clone(), Some(Type::Tuple(vec![Type::I32, Type::Bool])));
        let Expr::Tuple(items) = program.arena.expr(*value) else {
            panic!("expected tuple literal");
        };
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn rustlike_parser_accepts_loop_expression_with_break_value() {
        let src = r#"
fn main() {
    let total: f64 = loop {
        break 1.0;
    };
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("loop expression should parse");
        let func = &program.functions[0];
        let Stmt::Let { value, .. } = program.arena.stmt(func.body[0]) else {
            panic!("expected let statement");
        };
        let Expr::Loop(loop_expr) = program.arena.expr(*value) else {
            panic!("expected loop expression");
        };
        assert_eq!(loop_expr.body.len(), 1);
        let Stmt::Break(Some(expr_id)) = program.arena.stmt(loop_expr.body[0]) else {
            panic!("expected break statement");
        };
        assert!(matches!(
            program.arena.expr(*expr_id),
            Expr::NumericLiteral(NumericLiteral::F64(_))
        ));
    }

    #[test]
    fn rustlike_parser_accepts_statement_loop_with_continue_and_bare_break() {
        let src = r#"
fn main() {
    loop {
        continue;
        break;
    }
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("statement loop should parse");
        let func = &program.functions[0];
        let Stmt::Loop { body } = program.arena.stmt(func.body[0]) else {
            panic!("expected statement loop");
        };
        assert_eq!(body.len(), 2);
        assert!(matches!(program.arena.stmt(body[0]), Stmt::Continue));
        assert!(matches!(program.arena.stmt(body[1]), Stmt::Break(None)));
    }

    #[test]
    fn rustlike_parser_accepts_half_open_range_literal() {
        let src = r#"
fn main() {
    let interval = 0..10;
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("half-open range literal should parse");
        let func = &program.functions[0];
        let Stmt::Let { value, .. } = program.arena.stmt(func.body[0]) else {
            panic!("expected let statement");
        };
        let Expr::Range(range_expr) = program.arena.expr(*value) else {
            panic!("expected range literal");
        };
        assert!(!range_expr.inclusive);
        assert!(matches!(
            program.arena.expr(range_expr.start),
            Expr::NumericLiteral(NumericLiteral::I32(0))
        ));
        assert!(matches!(
            program.arena.expr(range_expr.end),
            Expr::NumericLiteral(NumericLiteral::I32(10))
        ));
    }

    #[test]
    fn rustlike_parser_accepts_closed_range_literal() {
        let src = r#"
fn main() {
    let interval = 1..=10;
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("closed range literal should parse");
        let func = &program.functions[0];
        let Stmt::Let { value, .. } = program.arena.stmt(func.body[0]) else {
            panic!("expected let statement");
        };
        let Expr::Range(range_expr) = program.arena.expr(*value) else {
            panic!("expected range literal");
        };
        assert!(range_expr.inclusive);
    }

    #[test]
    fn rustlike_parser_accepts_bare_break_in_statement_loop_only() {
        let src = r#"
fn main() {
    loop {
        break;
    }
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("bare break in statement loop should parse");
        let func = &program.functions[0];
        let Stmt::Loop { body } = program.arena.stmt(func.body[0]) else {
            panic!("expected statement loop");
        };
        assert!(matches!(program.arena.stmt(body[0]), Stmt::Break(None)));
    }

    #[test]
    fn rustlike_parser_accepts_ufcs_method_call_sugar() {
        let src = r#"
fn scale(value: f64, factor: f64) -> f64 = value * factor;

fn main() {
    let total: f64 = 2.0.scale(3.0);
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("UFCS method-call sugar should parse");
        let func = &program.functions[1];
        let Stmt::Let { value, .. } = program.arena.stmt(func.body[0]) else {
            panic!("expected let statement");
        };
        let Expr::Call(name, args) = program.arena.expr(*value) else {
            panic!("expected call expression");
        };
        assert_eq!(program.arena.symbol_name(*name), "scale");
        assert_eq!(args.len(), 2);
        assert!(matches!(
            program.arena.expr(args[0].value),
            Expr::NumericLiteral(NumericLiteral::F64(_))
        ));
    }

    #[test]
    fn rustlike_parser_accepts_ufcs_method_call_with_named_arguments() {
        let src = r#"
fn clamp(value: f64, min: f64, max: f64) -> f64 = value;

fn main() {
    let total: f64 = 2.0.clamp(min = 0.0, max = 10.0);
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("UFCS named-argument call should parse");
        let func = &program.functions[1];
        let Stmt::Let { value, .. } = program.arena.stmt(func.body[0]) else {
            panic!("expected let statement");
        };
        let Expr::Call(_, args) = program.arena.expr(*value) else {
            panic!("expected call expression");
        };
        assert_eq!(args.len(), 3);
        assert!(args[0].name.is_none());
        assert!(args[1].name.is_some());
        assert!(args[2].name.is_some());
    }

    #[test]
    fn rustlike_parser_accepts_postfix_field_access_without_parentheses() {
        let src = r#"
fn abs(value: f64) -> f64 = value;

fn main() {
    let total: f64 = 2.0.abs;
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("postfix field access should parse");
        let main = &program.functions[1];
        let Stmt::Let { value, .. } = program.arena.stmt(main.body[0]) else {
            panic!("expected let statement");
        };
        let Expr::RecordField(field_expr) = program.arena.expr(*value) else {
            panic!("expected record field access");
        };
        assert_eq!(program.arena.symbol_name(field_expr.field), "abs");
    }

    #[test]
    fn logos_parser_smoke() {
        let src = r#"
Entity Sensor:
    state val: quad

Law "CheckSignal" [priority 10]:
    When Sensor.val == T ->
        Log.emit("Signal OK")
"#;
        let a = parse_logos_with_profile(src, &ParserProfile::foundation_default())
            .expect("frontend logos");
        assert_eq!(a.entities.len(), 1);
        assert_eq!(a.laws.len(), 1);
    }

    #[test]
    fn strict_profile_rejects_f64_surface() {
        let profile = ParserProfile::default();
        let err = parse_rustlike_with_profile("fn main() -> f64 { return 1.5; }", &profile)
            .expect_err("strict profile must reject f64");

        assert_eq!(err.kind(), FrontendErrorKind::PolicyViolation);
        assert!(err.message.contains("f64"));
    }

    #[test]
    fn strict_profile_rejects_logos_surface() {
        let profile = ParserProfile::default();
        let src = r#"
Law "L" [priority 1]:
    When true -> System.recovery()
"#;
        let err =
            parse_logos_with_profile(src, &profile).expect_err("strict profile must reject logos");

        assert_eq!(err.kind(), FrontendErrorKind::PolicyViolation);
        assert!(err.message.contains("Logos surface"));
    }

    // --- M9.1 Wave 2: generic syntax admission tests ---

    #[test]
    fn generic_function_type_params_are_parsed_and_stored() {
        let src = r#"
fn identity<T>(x: T) -> T { return x; }
fn main() { return; }
"#;
        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("generic function should parse");
        let identity = &program.functions[0];
        assert_eq!(identity.type_params.len(), 1);
        let tp_name = program.arena.symbol_name(identity.type_params[0]);
        assert_eq!(tp_name, "T");
        // param type must be TypeVar, not Record
        assert!(matches!(identity.params[0].1, Type::TypeVar(_)));
        // return type must be TypeVar
        assert!(matches!(identity.ret, Type::TypeVar(_)));
    }

    #[test]
    fn generic_function_two_type_params_are_parsed() {
        let src = r#"
fn pair<A, B>(a: A, b: B) -> A { return a; }
fn main() { return; }
"#;
        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("two-param generic function should parse");
        let f = &program.functions[0];
        assert_eq!(f.type_params.len(), 2);
        assert!(matches!(f.params[0].1, Type::TypeVar(_)));
        assert!(matches!(f.params[1].1, Type::TypeVar(_)));
    }

    #[test]
    fn generic_record_type_params_are_parsed_and_stored() {
        let src = r#"
record Box<T> {
    value: T,
}
fn main() { return; }
"#;
        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("generic record should parse");
        let rec = &program.records[0];
        assert_eq!(rec.type_params.len(), 1);
        assert_eq!(program.arena.symbol_name(rec.type_params[0]), "T");
        assert!(matches!(rec.fields[0].ty, Type::TypeVar(_)));
    }

    #[test]
    fn generic_enum_type_params_are_parsed_and_stored() {
        let src = r#"
enum Maybe<T> {
    Some(T),
    None,
}
fn main() { return; }
"#;
        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("generic enum should parse");
        let adt = &program.adts[0];
        assert_eq!(adt.type_params.len(), 1);
        assert_eq!(program.arena.symbol_name(adt.type_params[0]), "T");
        // Some(T) payload should be TypeVar
        assert!(matches!(adt.variants[0].payload[0], Type::TypeVar(_)));
    }

    #[test]
    fn type_var_scope_does_not_leak_between_declarations() {
        let src = r#"
fn with_t<T>(x: T) -> T { return x; }
fn no_t(x: i32) -> i32 { return x; }
fn main() { return; }
"#;
        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("should parse");
        // second function has no type params
        assert!(program.functions[1].type_params.is_empty());
        // its param type must be I32, not TypeVar
        assert!(matches!(program.functions[1].params[0].1, Type::I32));
    }

    #[test]
    fn non_generic_function_retains_empty_type_params() {
        let src = r#"
fn add(a: i32, b: i32) -> i32 { return a; }
fn main() { return; }
"#;
        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("non-generic function should parse");
        assert!(program.functions[0].type_params.is_empty());
        assert!(matches!(program.functions[0].params[0].1, Type::I32));
    }

    #[test]
    fn langle_rangle_tokens_lex_correctly() {
        let tokens = crate::lexer::lex_tokens("fn foo<T>() {}").expect("should lex");
        let kinds: Vec<_> = tokens.iter().map(|t| t.kind).collect();
        assert!(kinds.contains(&TokenKind::LAngle));
        assert!(kinds.contains(&TokenKind::RAngle));
    }

    #[test]
    fn rustlike_parser_accepts_i32_relational_operator_surface() {
        let src = r#"
fn main() {
    let ok: bool = 3 >= 2;
    return;
}
"#;
        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("i32 relational expression should parse");
        let func = &program.functions[0];
        let Stmt::Let {
            ty: Some(Type::Bool),
            value,
            ..
        } = program.arena.stmt(func.body[0])
        else {
            panic!("expected bool let binding");
        };
        assert!(matches!(
            program.arena.expr(*value),
            Expr::Binary(_, BinaryOp::Ge, _)
        ));
    }

    #[test]
    fn rustlike_parser_gives_relational_precedence_between_additive_and_equality() {
        let src = r#"
fn main() {
    let ok: bool = 1 + 2 < 4 == true;
    return;
}
"#;
        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("relational precedence should parse");
        let func = &program.functions[0];
        let Stmt::Let {
            ty: Some(Type::Bool),
            value,
            ..
        } = program.arena.stmt(func.body[0])
        else {
            panic!("expected bool let binding");
        };
        let Expr::Binary(lhs, BinaryOp::Eq, rhs) = program.arena.expr(*value) else {
            panic!("expected equality at the top level");
        };
        assert!(matches!(
            program.arena.expr(*lhs),
            Expr::Binary(_, BinaryOp::Lt, _)
        ));
        assert!(matches!(program.arena.expr(*rhs), Expr::BoolLiteral(true)));
    }

    #[test]
    fn trait_decl_with_one_method_sig_is_parsed() {
        let src = r#"
trait Display {
    fn fmt(self: i32) -> i32;
}
"#;
        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("trait declaration should parse");
        assert_eq!(program.traits.len(), 1);
        assert_eq!(program.impls.len(), 0);
        let t = &program.traits[0];
        assert_eq!(t.methods.len(), 1);
        assert_eq!(t.type_params.len(), 0);
        let method = &t.methods[0];
        assert_eq!(method.params.len(), 1);
        assert_eq!(method.ret, Type::I32);
    }

    #[test]
    fn trait_decl_with_multiple_method_sigs_is_parsed() {
        let src = r#"
trait Printable {
    fn format(self: i32) -> i32;
    fn debug(self: i32) -> i32;
}
"#;
        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("multi-method trait should parse");
        assert_eq!(program.traits[0].methods.len(), 2);
    }

    #[test]
    fn impl_decl_with_one_method_body_is_parsed() {
        let src = r#"
trait Display {
    fn fmt(self: i32) -> i32;
}
record MyNum {
    value: i32,
}
impl Display for MyNum {
    fn fmt(self: i32) -> i32 {
        return self;
    }
}
"#;
        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("impl block should parse");
        assert_eq!(program.impls.len(), 1);
        let imp = &program.impls[0];
        assert_eq!(imp.methods.len(), 1);
        assert_eq!(imp.type_params.len(), 0);
    }

    #[test]
    fn iterable_trait_and_impl_surface_parse_on_current_main() {
        let src = r#"
trait Iterable {
    fn next(self: Numbers) -> Option(i32);
}

record Numbers {
    current: i32,
}

impl Iterable for Numbers {
    fn next(self: Numbers) -> Option(i32) {
        return Option::None;
    }
}

fn main() {
    return;
}
"#;

        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("Iterable trait and impl surface should parse");
        assert_eq!(program.traits.len(), 1);
        assert_eq!(program.impls.len(), 1);
        assert_eq!(
            program.arena.symbol_name(program.traits[0].name),
            "Iterable"
        );
        assert_eq!(
            program.arena.symbol_name(program.impls[0].trait_name),
            "Iterable"
        );
    }

    #[test]
    fn trait_method_self_type_parses_as_owner_layer_marker() {
        let src = r#"
trait Iterable {
    fn next(self: Self, index: i32) -> Option(i32);
}
"#;
        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("trait method Self type should parse");
        let self_symbol = program
            .arena
            .symbol_to_id
            .get("Self")
            .copied()
            .expect("Self symbol must be interned");
        let method = &program.traits[0].methods[0];
        assert_eq!(method.params[0].1, Type::TypeVar(self_symbol));
        assert_eq!(method.params[1].1, Type::I32);
    }

    #[test]
    fn impl_method_self_type_is_anchored_to_impl_target() {
        let src = r#"
trait Iterable {
    fn next(self: Self, index: i32) -> Option(i32);
}

record Numbers {
    current: i32,
}

impl Iterable for Numbers {
    fn next(self: Self, index: i32) -> Option(i32) {
        let _ = index;
        return Option::None;
    }
}
"#;
        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("impl method Self type should parse");
        let numbers = program.impls[0].for_type;
        let method = &program.impls[0].methods[0];
        assert_eq!(method.params[0].1, Type::Record(numbers));
        assert_eq!(method.params[1].1, Type::I32);
    }

    #[test]
    fn function_with_trait_bound_is_parsed() {
        let src = r#"
fn print_all<T: Display>(x: T) -> i32 {
    return 0;
}
"#;
        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("function with trait bound should parse");
        let func = &program.functions[0];
        assert_eq!(func.type_params.len(), 1);
        assert_eq!(func.trait_bounds.len(), 1);
        assert_eq!(func.trait_bounds[0].param, func.type_params[0]);
    }

    #[test]
    fn function_with_multiple_type_params_mixed_bounds_is_parsed() {
        let src = r#"
fn apply<T: Eq, U>(x: T, y: U) -> i32 {
    return 0;
}
"#;
        let program = parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
            .expect("mixed-bound function should parse");
        let func = &program.functions[0];
        assert_eq!(func.type_params.len(), 2);
        assert_eq!(func.trait_bounds.len(), 1);
    }

    // M9.4 Wave 2 — richer pattern surface parser admission

    fn parse_src(src: &str) -> Result<Program, crate::FrontendError> {
        parse_rustlike_with_profile(src, &ParserProfile::foundation_default())
    }

    #[test]
    fn wildcard_match_pattern_is_parsed() {
        let src = r#"
fn main() -> i32 {
    let v = match Color::Red {
        _ => { 0 }
    };
    return v;
}
"#;
        let p = parse_src(src).expect("wildcard match pattern should parse");
        assert!(!p.functions[0].body.is_empty());
    }

    #[test]
    fn or_pattern_two_alternatives_is_parsed() {
        let src = r#"
fn main() -> i32 {
    let v = match Color::Red {
        Color::Red | Color::Blue => { 1 }
    };
    return v;
}
"#;
        let p = parse_src(src).expect("or-pattern should parse");
        assert!(!p.functions[0].body.is_empty());
    }

    #[test]
    fn or_pattern_three_alternatives_is_parsed() {
        let src = r#"
fn main() -> i32 {
    let v = match Color::Red {
        Color::Red | Color::Blue | Color::Green => { 1 }
    };
    return v;
}
"#;
        let p = parse_src(src).expect("three-way or-pattern should parse");
        assert!(!p.functions[0].body.is_empty());
    }

    #[test]
    fn int_range_inclusive_pattern_is_parsed() {
        let src = r#"
fn main() -> i32 {
    let v = match Color::Red {
        1..=5 => { 1 }
    };
    return v;
}
"#;
        let p = parse_src(src).expect("inclusive range pattern should parse");
        assert!(!p.functions[0].body.is_empty());
    }

    #[test]
    fn int_range_exclusive_pattern_is_parsed() {
        let src = r#"
fn main() -> i32 {
    let v = match Color::Red {
        0..10 => { 2 }
    };
    return v;
}
"#;
        let p = parse_src(src).expect("exclusive range pattern should parse");
        assert!(!p.functions[0].body.is_empty());
    }

    #[test]
    fn nested_tuple_destructuring_is_parsed() {
        let src = r#"
fn main() -> i32 {
    let (a, (b, c)) = pair;
    let _ = a;
    let _ = b;
    let _ = c;
    return 0;
}
"#;
        // Parser should admit the shape without panicking. Typecheck may reject.
        let _ = parse_src(src);
    }

    #[test]
    fn if_let_adt_pattern_is_parsed() {
        let src = r#"
fn main() -> i32 {
    let r = if let Color::Red = x { 1 } else { 0 };
    return r;
}
"#;
        // Parser should admit the shape. Typecheck stub will reject at Wave 3.
        let _ = parse_src(src);
    }

    #[test]
    fn range_pattern_missing_end_rejects() {
        let src = r#"
fn main() -> i32 {
    let v = match Color::Red {
        1.. => 1
    };
    return v;
}
"#;
        let err = parse_src(src).expect_err("range pattern missing end bound must reject");
        assert!(
            err.message.contains("integer literal") || err.message.contains("range"),
            "unexpected error: {}",
            err.message
        );
    }
}
