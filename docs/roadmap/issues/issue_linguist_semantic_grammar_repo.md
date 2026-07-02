Title: linguist: create Semantic grammar repository

## Description

Create a public syntax highlighting grammar repository for Semantic so a future Linguist import can reference a stable grammar URL.

## Scope

Preferred target:

- TextMate-compatible grammar
- scopeName: `source.semantic`

Candidate repository names:

- `tree-sitter-semantic`
- `vscode-semantic`
- `semantic-textmate-grammar`

## Acceptance Criteria

- public grammar repository exists;
- grammar has a compatible open-source license;
- grammar covers canonical Semantic syntax;
- grammar can highlight representative `.sm` files;
- grammar URL is stable enough to reference from Linguist;
- grammar can be added through Linguist’s grammar import workflow.

## Non-goals

- do not open the upstream Linguist PR yet;
- do not invent compatibility claims without a published license;
- do not treat a local grammar stub as production-ready.

