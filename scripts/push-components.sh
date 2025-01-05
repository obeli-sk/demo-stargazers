#!/usr/bin/env bash

# Pushes all WASM components to the Docker Hub and updates obelisk-oci.toml

set -exuo pipefail
cd "$(dirname "$0")/.."

TAG="$1"
TOML_FILE="obelisk-oci.toml"

push() {
    RELATIVE_PATH=$1
    FILE_NAME_WITHOUT_EXT=$(basename "$RELATIVE_PATH" | sed 's/\.[^.]*$//')
    # Define the OCI location
    PREFIX="docker.io/getobelisk/demo_stargazers"
    OCI_LOCATION="${PREFIX}_${FILE_NAME_WITHOUT_EXT}:${TAG}"
    # Push the WASM component and capture the output
    echo "Pushing ${RELATIVE_PATH} to ${OCI_LOCATION}..."
    OUTPUT=$(obelisk client component push "$RELATIVE_PATH" "$OCI_LOCATION")

    # Replace the old location with the actual OCI location
    sed -i -E "/name = \"${FILE_NAME_WITHOUT_EXT}\"/{n;s|location\.oci = \".*\"|location.oci = \"${OUTPUT}\"|}" "$TOML_FILE"
}

# Make sure all components are fresh
cargo build

push "target/wasm32-wasip2/release/activity_llm_chatgpt.wasm"
push "target/wasm32-wasip2/release/activity_account_github.wasm"
push "target/wasm32-wasip2/release/activity_db_turso.wasm"
push "target/wasm32-unknown-unknown/release/workflow.wasm"
push "target/wasm32-wasip2/release/webhook.wasm"

echo "All components pushed and TOML file updated successfully."
