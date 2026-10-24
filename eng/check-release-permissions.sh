#!/bin/bash
# Fails if the release token in GITHUB_TOKEN can't push to the repository or publish releases,
# checked the way semantic-release does before releasing. Nothing is pushed.
set -uo pipefail

if [ -z "${GITHUB_TOKEN:-}" ]; then
  echo "the release token is not set"
  exit 1
fi
REPOSITORY=${GITHUB_REPOSITORY:?GITHUB_REPOSITORY is not set}

# Pushing needs write access to the repository; a dry run to an unused branch only checks it.
if ! git push --dry-run --no-verify \
  "https://x-access-token:${GITHUB_TOKEN}@github.com/${REPOSITORY}.git" \
  "HEAD:refs/heads/release-permission-check" > /dev/null 2>&1; then
  echo "the release token can't push to ${REPOSITORY}"
  exit 1
fi

# Publishing a release needs push permission on the repository.
if [ "$(GH_TOKEN=$GITHUB_TOKEN gh api "repos/${REPOSITORY}" --jq .permissions.push 2>/dev/null)" != "true" ]; then
  echo "the release token has no push permission on ${REPOSITORY}"
  exit 1
fi

echo "the release token can push and publish releases on ${REPOSITORY}"
