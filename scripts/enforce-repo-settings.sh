#!/usr/bin/env bash
# Idempotently enforce branch protection on the default branch via Rulesets.
# Falls back to legacy branch protection if Rulesets is unavailable.
#
# Note: GitHub's Rulesets API currently does not accept the `merge_queue`
# rule type via REST in a stable way. Enable Merge Queue via the GitHub UI:
#   Settings → Rules → main-protection → Add rule → "Require merge queue"
#
# Usage:
#   ./scripts/enforce-repo-settings.sh
#   REPO=owner/name ./scripts/enforce-repo-settings.sh

set -euo pipefail

REPO="${REPO:-$(gh repo view --json nameWithOwner --jq .nameWithOwner)}"
BRANCH="${BRANCH:-main}"
RULESET_NAME="${RULESET_NAME:-${BRANCH}-protection}"
REQUIRED_CHECK="${REQUIRED_CHECK:-Check}"

echo "==> Repo:    $REPO"
echo "==> Branch:  $BRANCH"
echo "==> Check:   $REQUIRED_CHECK"

RULESET_PAYLOAD=$(cat <<JSON
{
  "name": "$RULESET_NAME",
  "target": "branch",
  "enforcement": "active",
  "conditions": {
    "ref_name": {
      "include": ["refs/heads/$BRANCH"],
      "exclude": []
    }
  },
  "rules": [
    { "type": "deletion" },
    { "type": "non_fast_forward" },
    { "type": "required_linear_history" },
    {
      "type": "required_status_checks",
      "parameters": {
        "strict_required_status_checks_policy": true,
        "required_status_checks": [
          { "context": "$REQUIRED_CHECK" }
        ]
      }
    }
  ]
}
JSON
)

LEGACY_PAYLOAD=$(cat <<JSON
{
  "required_status_checks": {
    "strict": true,
    "contexts": ["$REQUIRED_CHECK"]
  },
  "enforce_admins": false,
  "required_pull_request_reviews": null,
  "restrictions": null,
  "required_linear_history": true,
  "allow_force_pushes": false,
  "allow_deletions": false,
  "required_conversation_resolution": true
}
JSON
)

apply_rulesets() {
  local list id
  list=$(gh api "repos/$REPO/rulesets" 2>/dev/null) || return 1
  id=$(jq -r ".[] | select(.name == \"$RULESET_NAME\") | .id" <<<"$list")
  if [ -n "$id" ] && [ "$id" != "null" ]; then
    echo "==> Rulesets: updating id=$id"
    gh api -X PUT "repos/$REPO/rulesets/$id" --input - <<<"$RULESET_PAYLOAD" >/dev/null
  else
    echo "==> Rulesets: creating"
    gh api -X POST "repos/$REPO/rulesets" --input - <<<"$RULESET_PAYLOAD" >/dev/null
  fi
}

apply_legacy() {
  echo "==> Legacy branch protection on $BRANCH"
  gh api -X PUT "repos/$REPO/branches/$BRANCH/protection" \
    -H "Accept: application/vnd.github+json" \
    --input - <<<"$LEGACY_PAYLOAD" >/dev/null
}

if apply_rulesets 2>/tmp/ruleset.err; then
  echo "==> Applied via Rulesets."
else
  echo "==> Rulesets failed, falling back:"
  cat /tmp/ruleset.err >&2
  apply_legacy
  echo "==> Applied legacy branch protection."
fi

echo
echo "==> Active rulesets:"
gh api "repos/$REPO/rulesets" --jq '.[] | {id, name, enforcement}'

echo
echo "==> To enable Merge Queue (recommended):"
echo "    https://github.com/$REPO/settings/rules → edit \"$RULESET_NAME\" → add \"Require merge queue\""
