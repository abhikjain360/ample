#!/bin/bash
set -e

APP_BUNDLE="$1"
FRAMEWORKS_DIR="$APP_BUNDLE/Contents/Frameworks"

mkdir -p "$FRAMEWORKS_DIR"

dylibbundler -od -b \
  -x "$APP_BUNDLE/Contents/MacOS/ample" \
  -d "$FRAMEWORKS_DIR" \
  -p @executable_path/../Frameworks/

echo "Bundled libmpv and dependencies into $FRAMEWORKS_DIR"
