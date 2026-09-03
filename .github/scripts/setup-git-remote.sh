#!/usr/bin/env bash
set -euo pipefail

if [ -z "${PAT_TOKEN:-}" ]; then
  echo "PAT_TOKEN is not set"
  exit 1
fi

REPO="${GITHUB_REPOSITORY:-sj4nes/ranger-arranger}"
git remote set-url origin "https://x-access-token:${PAT_TOKEN}@github.com/${REPO}.git"
git remote -v
