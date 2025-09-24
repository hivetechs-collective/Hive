#!/usr/bin/env bash
# Configure GitHub repo secrets, environment secrets, and branch protections.
# Requires: gh CLI (https://cli.github.com/), authenticated user with admin rights on the repo.

set -euo pipefail

if ! command -v gh >/dev/null 2>&1; then
  echo "❌ GitHub CLI (gh) not found. Install via 'brew install gh' or see https://cli.github.com/." >&2
  exit 1
fi

echo "🔐 Checking GitHub authentication..."
if ! gh auth status >/dev/null 2>&1; then
  echo "❌ gh CLI is not logged in. Run 'gh auth login' and rerun this script." >&2
  exit 1
fi

default_repo="$(git config --get remote.origin.url | sed -E 's#(git@|https://)([^/:]+)[:/]([^/]+)/([^/.]+)(\.git)?#\3/\4#')"
read -rp "GitHub repository [${default_repo}]: " repo
repo="${repo:-$default_repo}"
if [[ -z "$repo" ]]; then
  echo "❌ Repository cannot be empty (format: owner/repo)." >&2
  exit 1
fi

owner="${repo%%/*}"
name="${repo#*/}"

read -rp "Release channel to protect (branch name) [master]: " branch
branch="${branch:-master}"

echo "\nEnter Cloudflare R2 credentials (these will be stored as GitHub secrets):"
read -rp "  R2 Account ID: " r2_account
read -rp "  R2 Access Key ID: " r2_access
read -srp "  R2 Secret Access Key: " r2_secret
echo

if [[ -z "$r2_account" || -z "$r2_access" || -z "$r2_secret" ]]; then
  echo "❌ All R2 fields are required." >&2
  exit 1
fi

set_secret() {
  local scope=$1 name=$2 value=$3
  if [[ "$scope" == "env" ]]; then
    gh secret set "$name" --repo "$repo" --env production --body "$value"
  else
    gh secret set "$name" --repo "$repo" --body "$value"
  fi
}

echo "\n📦 Creating/Updating repository secrets..."
set_secret repo R2_ACCOUNT_ID "$r2_account"
set_secret repo R2_ACCESS_KEY_ID "$r2_access"
set_secret repo R2_SECRET_ACCESS_KEY "$r2_secret"

echo "\n🌐 Ensuring 'production' environment exists..."
gh api \
  --method PUT \
  "repos/$repo/environments/production" >/dev/null

echo "\n📦 Adding secrets to 'production' environment..."
set_secret env R2_ACCOUNT_ID "$r2_account"
set_secret env R2_ACCESS_KEY_ID "$r2_access"
set_secret env R2_SECRET_ACCESS_KEY "$r2_secret"

echo "\n🛡️ Applying branch protection to '$branch'..."
status_checks=("CI / Backend (Rust)" "CI / Electron Unit & Lint" "CI / CI Summary")
status_json=$(printf '"%s",' "${status_checks[@]}")
status_json="[${status_json%,}]"

gh api \
  --method PUT \
  "repos/$repo/branches/$branch/protection" \
  --input - <<JSON
{
  "required_status_checks": {
    "strict": true,
    "contexts": $status_json
  },
  "enforce_admins": true,
  "required_pull_request_reviews": {
    "required_approving_review_count": 1,
    "dismiss_stale_reviews": true,
    "require_code_owner_reviews": false
  },
  "restrictions": null,
  "required_linear_history": true,
  "allow_force_pushes": false,
  "allow_deletions": false
}
JSON

cat <<SUMMARY

✅ Configuration complete for $repo
  • Repository + environment secrets (R2 credentials)
  • 'production' environment ensured
  • Branch protection applied to '$branch'

Next steps (manual):
  - Update .github/CODEOWNERS with the correct teams.
  - (Optional) Add reviewers to the 'production' environment via the GitHub UI.

SUMMARY
