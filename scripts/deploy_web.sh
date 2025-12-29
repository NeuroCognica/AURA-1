#!/usr/bin/env bash
set -euo pipefail

# Pushes frontend/build to the web repo using git subtree.
# Requirements:
# - WEB_REPO (e.g., git@github.com:NeuroCognica/AURA-1-web.git)
# - WEB_REPO_BRANCH (default: main)

WEB_REPO="${WEB_REPO:-}"
WEB_REPO_BRANCH="${WEB_REPO_BRANCH:-main}"
BUILD_DIR="${BUILD_DIR:-frontend/build}"

if [[ -z "$WEB_REPO" ]]; then
  echo "WEB_REPO is required (e.g., git@github.com:NeuroCognica/AURA-1-web.git)" >&2
  exit 1
fi

if [[ ! -d "$BUILD_DIR" ]]; then
  echo "Build directory '$BUILD_DIR' not found. Run 'npm run build' in frontend/ first." >&2
  exit 1
fi

git subtree push --prefix "$BUILD_DIR" "$WEB_REPO" "$WEB_REPO_BRANCH"
