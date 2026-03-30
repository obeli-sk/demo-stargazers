# Build all components (Rust, JS)
build: rust js

# Build Rust components
rust:
	(cd activity/github/impl && cargo build --release)
	(cd activity/db/turso && cargo build --release)
	(cd activity/llm/openai && cargo build --release)
	(cd webhook/webhook-rs && cargo build --release)
	(cd workflow/stargazers/workflow-rs && cargo build --release)

# Build JavaScript components
js:
	./scripts/build-components-js.sh

serve:
	obelisk server run --config ./obelisk-local.toml

test-unit:
	./scripts/test-unit.sh
test-integration:
	./scripts/test-integration.sh
test-e2e: rust
	./scripts/test-e2e.sh ./obelisk-local.toml truncate
test-e2e-js: js
	./scripts/test-e2e.sh ./obelisk-local-js-all.toml truncate
