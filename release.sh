#!/bin/bash
# Script: release-prep.sh
# Usage: ./release-prep.sh
#
# Reads version from version.txt
# Copies files from ./mold/ to ./
# Replaces %version% placeholder with actual version

set -euo pipefail

# Read version
VERSION=$(cat version.txt | tr -d ' \n')
if [ -z "$VERSION" ]; then
    echo "ERROR: version.txt is empty"
    exit 1
fi

echo "Preparing release for version: $VERSION"

# Check if mold directory exists
if [ ! -d "./mold" ]; then
    echo "ERROR: ./mold directory not found"
    exit 1
fi

# Copy and replace
find ./mold -type f | while read -r file; do
    # Get relative path
    rel_path="${file#./mold/}"
    target="./$rel_path"

    # Create target directory if needed
    mkdir -p "$(dirname "$target")"

    # Copy and replace %version%
    sed "s/%version%/$VERSION/g" "$file" > "$target"

    echo "  $rel_path -> $target"
done

echo "Done. Version $VERSION applied to all files."

## 5. Закоммить и затегать
./make-release.sh && \
./push.sh "Release v$VERSION" && \
git tag "v$VERSION" && \
git push origin main "v$VERSION"
