#!/usr/bin/env bash
# Release script for rp2350-adc-protobuf
# Usage: ./scripts/release.sh [version]
# Example: ./scripts/release.sh 0.2.0

set -euo pipefail

VERSION="${1:-}"

if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 0.2.0"
    echo ""
    echo "Current version: $(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')"
    exit 1
fi

# Validate version format (semver)
if ! echo "$VERSION" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+$'; then
    echo "Error: Version must be in semver format (e.g., 1.2.3)"
    exit 1
fi

TAG="v${VERSION}"

# Check for uncommitted changes
if ! git diff --quiet || ! git diff --cached --quiet; then
    echo "Error: You have uncommitted changes. Please commit or stash them first."
    exit 1
fi

# Check we're on main branch (default is main or master)
BRANCH=$(git branch --show-current)
if [ "$BRANCH" != "main" ] && [ "$BRANCH" != "master" ]; then
    echo "Warning: You are on branch '$BRANCH', not 'main' or 'master'."
    read -rp "Continue anyway? [y/N] " confirm
    if [ "$confirm" != "y" ] && [ "$confirm" != "Y" ]; then
        exit 1
    fi
fi

# Check tag doesn't already exist
if git tag -l "$TAG" | grep -q "$TAG"; then
    echo "Error: Tag $TAG already exists."
    exit 1
fi

echo "==> Updating Cargo.toml version to $VERSION"
sed -i "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml

echo "==> Running cargo check for target thumbv8m.main-none-eabihf..."
# Ensure rustup PATH is included if run outside of sourced environment
export PATH="$HOME/.cargo/bin:$PATH"
cargo check --target thumbv8m.main-none-eabihf

echo "==> Updating Cargo.lock..."
cargo update --workspace

echo "==> Committing version bump"
git add Cargo.toml Cargo.lock
git commit --allow-empty -m "Release $TAG"

echo "==> Creating tag $TAG"
git tag "$TAG"

echo ""
echo "Done! Release $TAG has been prepared and committed locally."
echo "To publish, run:"
echo "  git push origin $BRANCH"
echo "  git push origin $TAG"
echo ""
echo "This will trigger the GitHub Actions release build."
