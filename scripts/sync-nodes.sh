#!/usr/bin/env bash
# Sync custom nodes from nodes.yaml
# Usage: ./scripts/sync-nodes.sh [install|update|status]
#
# Run inside the ComfyUI container or with proper paths.

set -euo pipefail

COMFYUI_DIR="${COMFYUI_DIR:-/opt/comfyui}"
NODES_DIR="$COMFYUI_DIR/custom_nodes"
SCRIPT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
YAML_FILE="$SCRIPT_DIR/nodes.yaml"

if [ ! -f "$YAML_FILE" ]; then
    echo "ERROR: nodes.yaml not found at $YAML_FILE"
    exit 1
fi

ACTION="${1:-status}"

# Parse nodes.yaml (simple grep-based, no yq dependency)
parse_nodes() {
    local name="" repo="" commit=""
    while IFS= read -r line; do
        case "$line" in
            *"- name:"*)
                name=$(echo "$line" | sed 's/.*- name: //' | tr -d ' ')
                ;;
            *"repo:"*)
                repo=$(echo "$line" | sed 's/.*repo: //' | tr -d ' ')
                ;;
            *"commit:"*)
                commit=$(echo "$line" | sed 's/.*commit: //' | tr -d ' ')
                if [ -n "$name" ] && [ -n "$repo" ]; then
                    echo "$name|$repo|$commit"
                fi
                name="" repo="" commit=""
                ;;
        esac
    done < "$YAML_FILE"
}

install_nodes() {
    echo "=== Installing missing nodes ==="
    parse_nodes | while IFS='|' read -r name repo commit; do
        if [ -d "$NODES_DIR/$name" ]; then
            echo "  [skip] $name (already exists)"
        else
            echo "  [install] $name from $repo"
            git clone "$repo" "$NODES_DIR/$name"
            if [ -n "$commit" ] && [ "$commit" != "n/a" ]; then
                git -C "$NODES_DIR/$name" checkout "$commit" 2>/dev/null || true
            fi
        fi
    done
    echo "Done. Restart ComfyUI to load new nodes."
}

update_nodes() {
    echo "=== Updating all nodes ==="
    parse_nodes | while IFS='|' read -r name repo commit; do
        if [ -d "$NODES_DIR/$name" ]; then
            echo "  [update] $name"
            git -C "$NODES_DIR/$name" pull --ff-only 2>/dev/null || echo "    WARNING: pull failed for $name"
        else
            echo "  [missing] $name — run 'install' first"
        fi
    done
    echo "Done. Restart ComfyUI to load updated nodes."
}

show_status() {
    echo "=== Node Status ==="
    printf "%-40s %-10s %-10s %s\n" "NAME" "EXPECTED" "CURRENT" "STATUS"
    echo "------------------------------------------------------------------------------------"
    parse_nodes | while IFS='|' read -r name repo commit; do
        if [ -d "$NODES_DIR/$name" ]; then
            current=$(git -C "$NODES_DIR/$name" rev-parse --short HEAD 2>/dev/null || echo "n/a")
            if [ "$current" = "$commit" ]; then
                status="OK"
            else
                status="DRIFT"
            fi
        else
            current="-"
            status="MISSING"
        fi
        printf "%-40s %-10s %-10s %s\n" "$name" "$commit" "$current" "$status"
    done
}

case "$ACTION" in
    install) install_nodes ;;
    update)  update_nodes ;;
    status)  show_status ;;
    *)
        echo "Usage: $0 [install|update|status]"
        exit 1
        ;;
esac
