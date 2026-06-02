#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROCESSORS_DIR="$SCRIPT_DIR"

OUTPUT_DIR="${1:-$HOME/.musicum/processors}"
# expand a literal leading ~ that survived quoting
OUTPUT_DIR="${OUTPUT_DIR/#\~/$HOME}"

mkdir -p "$OUTPUT_DIR"

case "$(uname -s)" in
  Darwin) EXT="dylib" ;;
  Linux)  EXT="so"    ;;
  *)      EXT="so"    ;;
esac

echo "Building processors (release)..."
(cd "$PROCESSORS_DIR" && cargo build --release)

echo "Copying .$EXT files to $OUTPUT_DIR ..."
count=0
for f in "$PROCESSORS_DIR/target/release/"*."$EXT"; do
  [ -f "$f" ] || continue   # skip if glob matched nothing
  [ -L "$f" ] && continue   # skip symlinks
  fname="$(basename "$f")"
  cp "$f" "$OUTPUT_DIR/$fname"
  echo "  copied: $fname -> $OUTPUT_DIR"
  count=$((count + 1))
done

echo "Done: $count processor(s) copied to $OUTPUT_DIR"
