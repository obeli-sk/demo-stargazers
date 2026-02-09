#!/usr/bin/env bash

# Pushes all WASM components to the Docker Hub and updates obelisk-oci.toml

set -exuo pipefail
cd "$(dirname "$0")/.."

TAG="$1"
TOML_FILE="obelisk-oci.toml"
PREFIX="docker.io/getobelisk/demo_stargazers_"

push() {
    RELATIVE_PATH=$1
    FILE_NAME_WITHOUT_EXT=$(basename "$RELATIVE_PATH" | sed 's/\.[^.]*$//')
    OCI_LOCATION="${PREFIX}${FILE_NAME_WITHOUT_EXT}:${TAG}"
    echo "Pushing ${RELATIVE_PATH} to ${OCI_LOCATION}..."
    OUTPUT=$(obelisk component push "$RELATIVE_PATH" "$OCI_LOCATION")

    # Replace the old location with the actual OCI location
    sed -i -E "/name = \"${FILE_NAME_WITHOUT_EXT}\"/{n;s|location = \".*\"|location = \"${OUTPUT}\"|}" "$TOML_FILE"
}

# Rebuild rust components
just rust

push "target/wasm32-wasip2/release/activity_llm_openai.wasm"
push "target/wasm32-wasip2/release/activity_github_impl.wasm"
push "target/wasm32-wasip2/release/activity_db_turso.wasm"
push "target/wasm32-unknown-unknown/release/workflow.wasm"
push "target/wasm32-wasip2/release/webhook.wasm"

echo "All components pushed and TOML file updated successfully."
