# Build all components (Rust, JS, Go)
build: rust js go

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

# Build Go components
go:
	./scripts/build-components-go.sh

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
test-e2e-go: go
	./scripts/test-e2e.sh ./obelisk-local-go-all.toml truncate
