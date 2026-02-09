#!/usr/bin/env bash

# Pushes all WASM components to the Docker Hub and updates obelisk-oci.toml

set -exuo pipefail
cd "$(dirname "$0")/.."

TAG="$1"
TOML_FILE="obelisk-oci.toml"
PREFIX="docker.io/getobelisk/demo_stargazers_"

push() {
    COMPONENT_TYPE=$1
    RELATIVE_PATH=$2
    
    FILE_NAME_WITHOUT_EXT=$(basename "$RELATIVE_PATH" | sed 's/\.[^.]*$//')
    OCI_LOCATION="${PREFIX}${FILE_NAME_WITHOUT_EXT}:${TAG}"
    echo "Pushing ${RELATIVE_PATH} to ${OCI_LOCATION}..."
    OUTPUT=$(obelisk component push "$RELATIVE_PATH" "$OCI_LOCATION")

    # Replace the old location with the actual OCI location
    obelisk component add ${COMPONENT_TYPE} ${OUTPUT} --name ${FILE_NAME_WITHOUT_EXT} -c $TOML_FILE
}

# Rebuild rust components
just rust

push activity_wasm "target/wasm32-wasip2/release/activity_llm_openai.wasm"
push activity_wasm "target/wasm32-wasip2/release/activity_github_impl.wasm"
push activity_wasm "target/wasm32-wasip2/release/activity_db_turso.wasm"
push workflow "target/wasm32-unknown-unknown/release/workflow.wasm"
push webhook_endpoint "target/wasm32-wasip2/release/webhook.wasm"

echo "All components pushed and TOML file updated successfully."
