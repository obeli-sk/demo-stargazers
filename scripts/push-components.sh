#!/usr/bin/env bash

# Pushes all WASM components to the Docker Hub and updates obelisk-oci.toml

set -exuo pipefail
cd "$(dirname "$0")/.."

TAG="$1"
PREFIX="oci://docker.io/getobelisk/demo_stargazers_"

push_component() {
    local LOCAL_DEPLOYMENT_TOML="$1"
    local COMPONENT_NAME="$2"

    OCI_LOCATION="${PREFIX}${COMPONENT_NAME}:${TAG}"
    obelisk component push --deployment "$LOCAL_DEPLOYMENT_TOML" "$COMPONENT_NAME" "$OCI_LOCATION"
}

push_and_update() {
    local LOCAL_DEPLOYMENT_TOML="$1"
    local COMPONENT_NAME="$2"
    shift 2
    DST_TOML_FILES=("$@")

    OCI_LOCATION=$(push_component "$LOCAL_DEPLOYMENT_TOML" "$COMPONENT_NAME")

    for DST_TOML_FILE in "${DST_TOML_FILES[@]}"; do
        obelisk component add --deployment "$DST_TOML_FILE" "$OCI_LOCATION" "$COMPONENT_NAME"
    done
}

just rust

push_and_update obelisk-local.toml activity_llm_openai obelisk-oci.toml
push_and_update obelisk-local.toml activity_github_impl obelisk-oci.toml
push_and_update obelisk-local.toml activity_db_turso obelisk-oci.toml
push_and_update obelisk-local.toml workflow obelisk-oci.toml
push_and_update obelisk-local.toml webhook obelisk-oci.toml

echo "All components pushed and TOML file updated successfully."
