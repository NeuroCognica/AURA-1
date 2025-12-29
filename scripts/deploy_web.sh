#!/usr/bin/env bash
# deploy_web.sh
# Manual helper to push the built frontend to a remote web repo using git subtree.
# Usage: ./scripts/deploy_web.sh <WEB_REPO> [branch] [build_dir]

set -euo pipefail

WEB_REPO=${1:-}
BRANCH=${2:-main}
BUILD_DIR=${3:-frontend/build}

if [ -z "$WEB_REPO" ]; then
  echo "Usage: $0 <WEB_REPO> [branch] [build_dir]"
  echo "Example: $0 NeuroCognica/AURA-1-web main frontend/build"
  exit 1
fi

if [ ! -d "$BUILD_DIR" ]; then
  echo "Build directory '$BUILD_DIR' not found. Run frontend build first." >&2
  exit 1
fi

echo "Preparing temporary branch for subtree push..."
TMP_BRANCH="deploy/web-$(date +%s)"

git checkout --orphan "$TMP_BRANCH"
git rm -rf . >/dev/null 2>&1 || true
mkdir -p .
cp -R "$BUILD_DIR"/* .
git add -A
git commit -m "chore: deploy frontend build ($(date -u))"

echo "Pushing to $WEB_REPO branch $BRANCH using subtree commit"
git push "https://github.com/$WEB_REPO.git" HEAD:$BRANCH --force

echo "Cleaning up"
git checkout -
git branch -D "$TMP_BRANCH" || true

echo "Deploy finished."
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
