# CI/CD Guidelines

## UI smoke tests
- The `ui-smoke` job runs automatically on pushes to `main`/`master` and whenever a pull request is tagged with the `ui-smoke` label.
- Before merging to `master`, add the label to your PR so the macOS Playwright suite builds the DMG and executes the smoke run.
- Keep the label until the job finishes; make the `ui-smoke` check required in branch protection to guarantee the smoke suite passes before merging.

## CodeQL / GitHub Advanced Security
- The CodeQL workflow now checks whether GitHub Advanced Security is enabled for the repository.
- If Advanced Security is disabled, the workflow skips gracefully and reminds you to enable it under **Settings → Code security and analysis**.
- Once enabled, the `CodeQL Security Scan` job will run automatically and can be marked as a required check for merges to `master`.
