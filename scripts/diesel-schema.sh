#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

usage() {
    cat <<'EOF'
Usage:
  scripts/diesel-schema.sh sync
  scripts/diesel-schema.sh diff <migration_name> <schema_key>

Commands:
  sync
      Refresh all Diesel schema files from the current database.

  diff <migration_name> <schema_key>
      Generate a migration from the difference between the configured
      schema file and the current database for one schema key.
      Valid schema keys: auth, blog, activity
EOF
}

sync_schema() {
    local key="$1"
    local output="$2"
    local tmp_file

    tmp_file="$(mktemp)"
    trap 'rm -f "$tmp_file"' RETURN

    diesel print-schema --config-file "$ROOT_DIR/diesel.toml" --schema-key "$key" > "$tmp_file"
    mv "$tmp_file" "$ROOT_DIR/$output"
    trap - RETURN
}

case "${1:-}" in
    sync)
        sync_schema auth "src/user/schemas.rs"
        sync_schema blog "src/blogs/schemas.rs"
        sync_schema activity "src/activitylog/schemas.rs"
        ;;
    diff)
        if [[ $# -ne 3 ]]; then
            usage
            exit 1
        fi
        diesel migration generate \
            --config-file "$ROOT_DIR/diesel.toml" \
            --schema-key "$3" \
            --diff-schema \
            "$2"
        ;;
    *)
        usage
        exit 1
        ;;
esac
