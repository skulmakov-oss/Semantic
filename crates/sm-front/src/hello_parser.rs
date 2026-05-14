use crate::lexer::lex_tokens;
use crate::types::{FrontendError, Token, TokenKind};
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelloFile {
    pub entry: HelloEntry,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelloEntry {
    pub name: String,
    pub body: Vec<HelloStmt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HelloStmt {
    State(HelloStateDecl),
    Require(HelloRequireQuadEq),
    Observe(HelloObserveText),
    Complete(HelloCompleteQuad),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HelloStateType {
    Quad,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelloStateDecl {
    pub name: String,
    pub ty: HelloStateType,
    pub value: HelloQuadLit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelloRequireQuadEq {
    pub state: String,
    pub value: HelloQuadLit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelloObserveText {
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelloCompleteQuad {
    pub value: HelloQuadLit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelloQuadLit {
    T,
    F,
    N,
    S,
}

pub fn parse_hello_file(input: &str) -> Result<HelloFile, FrontendError> {
    let tokens = lex_tokens(input)?;
    let mut parser = HelloParser { tokens, idx: 0 };
    parser.parse_file()
}

struct HelloParser {
    tokens: Vec<Token>,
    idx: usize,
}

impl HelloParser {
    fn parse_file(&mut self) -> Result<HelloFile, FrontendError> {
        self.skip_layout();
        let entry = self.parse_entry()?;
        self.skip_layout();
        if !self.is_eof() {
            return Err(self.error_here("unexpected trailing tokens after Hello entry"));
        }
        Ok(HelloFile { entry })
    }

    fn parse_entry(&mut self) -> Result<HelloEntry, FrontendError> {
        if self.peek_kind() == Some(TokenKind::KwFn) {
            return Err(self.error_here(
                "legacy Hello syntax is unsupported in isolated parser path; expected `entry`",
            ));
        }
        self.expect_ident_text("entry", "expected `entry` to begin Hello file")?;
        let name = self.expect_ident("expected Hello entry name after `entry`")?;
        self.expect_kind(TokenKind::LBrace, "expected `{` after Hello entry name")?;

        let mut body = Vec::new();
        let mut saw_complete = false;
        loop {
            self.skip_layout();
            if self.eat_kind(TokenKind::RBrace) {
                break;
            }
            if self.is_eof() {
                return Err(self.error_here("expected `}` to close Hello entry body"));
            }
            if saw_complete {
                return Err(self.error_here("statement after complete is not allowed"));
            }
            let stmt = self.parse_stmt()?;
            if matches!(stmt, HelloStmt::Complete(_)) {
                saw_complete = true;
            }
            body.push(stmt);
        }

        Ok(HelloEntry { name, body })
    }

    fn parse_stmt(&mut self) -> Result<HelloStmt, FrontendError> {
        if self.eat_ident_text("state") {
            return self.parse_state_decl();
        }
        if self.eat_ident_text("require") {
            return self.parse_require_decl();
        }
        if self.eat_ident_text("observe") {
            return self.parse_observe_stmt();
        }
        if self.eat_ident_text("complete") {
            return self.parse_complete_stmt();
        }
        if self.peek_kind() == Some(TokenKind::KwFn) {
            return Err(self.error_here(
                "legacy Hello syntax is unsupported in isolated parser path; expected `entry`",
            ));
        }
        Err(self
            .error_here("expected Hello statement: `state`, `require`, `observe`, or `complete`"))
    }

    fn parse_state_decl(&mut self) -> Result<HelloStmt, FrontendError> {
        let name = self.expect_ident("expected state name after `state`")?;
        self.expect_kind(TokenKind::Colon, "expected `:` after state name")?;
        self.expect_kind(TokenKind::TyQuad, "expected `quad` in state declaration")?;
        self.expect_kind(TokenKind::Assign, "expected `=` in state declaration")?;
        let value = self.parse_quad_lit("expected quad literal in state declaration")?;
        self.expect_kind(TokenKind::Semi, "expected `;` after state declaration")?;
        Ok(HelloStmt::State(HelloStateDecl {
            name,
            ty: HelloStateType::Quad,
            value,
        }))
    }

    fn parse_require_decl(&mut self) -> Result<HelloStmt, FrontendError> {
        let state = self.expect_ident("expected state symbol after `require`")?;
        self.expect_kind(TokenKind::EqEq, "expected `==` after required state symbol")?;
        let value = self.parse_quad_lit("expected quad literal in requirement")?;
        self.expect_kind(TokenKind::Semi, "expected `;` after requirement")?;
        Ok(HelloStmt::Require(HelloRequireQuadEq { state, value }))
    }

    fn parse_observe_stmt(&mut self) -> Result<HelloStmt, FrontendError> {
        let text = self.expect_string_literal("expected text literal after `observe`")?;
        self.expect_kind(TokenKind::Semi, "expected `;` after observation")?;
        Ok(HelloStmt::Observe(HelloObserveText { text }))
    }

    fn parse_complete_stmt(&mut self) -> Result<HelloStmt, FrontendError> {
        let value = self.parse_quad_lit("expected quad literal after `complete`")?;
        self.expect_kind(TokenKind::Semi, "expected `;` after completion")?;
        Ok(HelloStmt::Complete(HelloCompleteQuad { value }))
    }

    fn parse_quad_lit(&mut self, message: &str) -> Result<HelloQuadLit, FrontendError> {
        let tok = self.next_token().ok_or_else(|| self.error_here(message))?;
        match tok.kind {
            TokenKind::QuadT => Ok(HelloQuadLit::T),
            TokenKind::QuadF => Ok(HelloQuadLit::F),
            TokenKind::QuadN => Ok(HelloQuadLit::N),
            TokenKind::QuadS => Ok(HelloQuadLit::S),
            _ => Err(FrontendError::syntax(tok.pos, message)),
        }
    }

    fn expect_ident(&mut self, message: &str) -> Result<String, FrontendError> {
        let tok = self.next_token().ok_or_else(|| self.error_here(message))?;
        if tok.kind == TokenKind::Ident {
            Ok(tok.text)
        } else {
            Err(FrontendError::syntax(tok.pos, message))
        }
    }

    fn expect_string_literal(&mut self, message: &str) -> Result<String, FrontendError> {
        let tok = self.next_token().ok_or_else(|| self.error_here(message))?;
        if tok.kind == TokenKind::String {
            Ok(tok.text)
        } else {
            Err(FrontendError::syntax(tok.pos, message))
        }
    }

    fn expect_ident_text(&mut self, expected: &str, message: &str) -> Result<(), FrontendError> {
        let tok = self.next_token().ok_or_else(|| self.error_here(message))?;
        if tok.kind == TokenKind::Ident && tok.text == expected {
            Ok(())
        } else {
            Err(FrontendError::syntax(tok.pos, message))
        }
    }

    fn expect_kind(&mut self, kind: TokenKind, message: &str) -> Result<(), FrontendError> {
        let tok = self.next_token().ok_or_else(|| self.error_here(message))?;
        if tok.kind == kind {
            Ok(())
        } else {
            Err(FrontendError::syntax(tok.pos, message))
        }
    }

    fn eat_kind(&mut self, kind: TokenKind) -> bool {
        if self.peek_kind() == Some(kind) {
            self.idx += 1;
            return true;
        }
        false
    }

    fn eat_ident_text(&mut self, expected: &str) -> bool {
        match self.peek_token() {
            Some(tok) if tok.kind == TokenKind::Ident && tok.text == expected => {
                self.idx += 1;
                true
            }
            _ => false,
        }
    }

    fn next_token(&mut self) -> Option<Token> {
        self.skip_layout();
        let tok = self.tokens.get(self.idx).cloned();
        if tok.is_some() {
            self.idx += 1;
        }
        tok
    }

    fn skip_layout(&mut self) {
        while matches!(
            self.tokens.get(self.idx).map(|t| t.kind),
            Some(TokenKind::Newline | TokenKind::Indent | TokenKind::Dedent)
        ) {
            self.idx += 1;
        }
    }

    fn peek_token(&mut self) -> Option<Token> {
        self.skip_layout();
        self.tokens.get(self.idx).cloned()
    }

    fn peek_kind(&mut self) -> Option<TokenKind> {
        self.peek_token().map(|tok| tok.kind)
    }

    fn is_eof(&mut self) -> bool {
        self.peek_token().is_none()
    }

    fn error_here(&mut self, message: impl Into<String>) -> FrontendError {
        let pos = self.peek_token().map(|tok| tok.pos).unwrap_or_else(|| {
            self.tokens
                .last()
                .map(|tok| tok.pos.saturating_add(tok.text.len()))
                .unwrap_or(0)
        });
        FrontendError::syntax(pos, message)
    }
}
