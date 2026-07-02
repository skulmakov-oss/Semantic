Title: linguist: prepare local validation plan for Semantic

## Description

Prepare and verify the local Linguist validation plan that will be used once the Semantic language patch is ready.

## Expected Validation

```bash
bundle exec ruby -Itest test/test_language.rb -n /test_find_by_extension/
bundle exec ruby -Itest test/test_language.rb -n /test_all_languages_have_a_language_id_set/
bundle exec ruby -Itest test/test_language.rb -n /test_all_languages_have_grammars/
bundle exec ruby -Itest test/test_classifier.rb
```

## Acceptance Criteria

- local Linguist checkout can run tests;
- targeted tests are defined and runnable;
- future Semantic patch passes the selected checks;
- classifier behavior is not regressed.

## Non-goals

- do not claim readiness without the grammar repo and usage evidence;
- do not open the upstream PR from this issue;
- do not invent a `language_id`.
