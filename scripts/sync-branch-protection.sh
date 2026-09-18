#!/usr/bin/env bash
# Applies branch protection for `main`, deriving required checks from PR workflows.

set -euo pipefail
cd "$(dirname "$0")/.."

BRANCH="${1:-main}"
REPO="$(gh repo view --json nameWithOwner -q .nameWithOwner)"

JOB_NAMES_JQ='
def cartesian(m):
  (m | to_entries) as $entries
  | if ($entries | length) == 0 then [{}]
    else
      reduce $entries[] as $e ([{}];
        [ .[] as $acc | $e.value[] | ($acc + {($e.key): .}) ]
      )
    end;

to_entries[] | . as $job
| ($job.value.name // $job.key) as $tmpl
| ($job.value.strategy.matrix // {} | with_entries(select(.key != "include" and .key != "exclude"))) as $matrix
| cartesian($matrix)[] as $combo
| reduce ($combo | to_entries[]) as $e ($tmpl;
    gsub("\\$\\{\\{\\s*matrix\\." + $e.key + "\\s*\\}\\}"; ($e.value | tostring))
  )
'

derive_required_checks() {
    local wf triggers_on_pr
    for wf in .github/workflows/*.yml .github/workflows/*.yaml; do
        [ -f "$wf" ] || continue

        triggers_on_pr="$(yq -o=json '.on' "$wf" | jq '
            if type == "array" then any(. == "pull_request")
            elif type == "object" then has("pull_request")
            else . == "pull_request"
            end
        ')"
        [ "$triggers_on_pr" = "true" ] || continue

        yq -o=json '.jobs' "$wf" | jq -r "$JOB_NAMES_JQ"
    done
}

REQUIRED_CHECKS="$(derive_required_checks)"
if [ -z "$REQUIRED_CHECKS" ]; then
    echo "No workflow jobs found that trigger on pull_request" >&2
    exit 1
fi

echo "Required status checks:"
echo "$REQUIRED_CHECKS" | sed 's/^/  - /'

CONTEXTS_JSON="$(echo "$REQUIRED_CHECKS" | jq -R . | jq -s .)"
PAYLOAD="$(jq -n --argjson contexts "$CONTEXTS_JSON" '{
    required_status_checks: {strict: true, contexts: $contexts},
    enforce_admins: false,
    required_pull_request_reviews: null,
    restrictions: null,
    required_linear_history: false,
    allow_force_pushes: false,
    allow_deletions: false,
    required_conversation_resolution: true
}')"

echo "Applying branch protection to $REPO@$BRANCH"
echo "$PAYLOAD" | gh api \
    --method PUT \
    -H "Accept: application/vnd.github+json" \
    "repos/$REPO/branches/$BRANCH/protection" \
    --input - \
    > /dev/null

echo "Current protection:"
gh api "repos/$REPO/branches/$BRANCH/protection" | jq .
