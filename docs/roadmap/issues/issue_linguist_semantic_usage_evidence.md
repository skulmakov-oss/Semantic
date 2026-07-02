Title: linguist: collect `.sm` usage evidence for Semantic

## Description

Collect and document GitHub Search evidence for the `.sm` extension before any Linguist submission.

## Evidence Query

```text
NOT is:fork path:*.sm Semantic
```

## Acceptance Criteria

- search URL is recorded;
- indexed result count is recorded;
- evidence excludes forks;
- distribution across unique `user/repo` combinations is checked manually;
- if results are dominated by `skulmakov-oss/Semantic` or related repos, that risk is documented honestly;
- current result does not overclaim readiness.

## Non-goals

- do not treat the query as proof by itself;
- do not open the upstream PR before usage evidence is recorded;
- do not claim the `.sm` threshold is met without a count and distribution check.

