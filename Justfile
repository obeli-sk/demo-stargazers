# Build all components (Rust, legacy ComponentizeJS, Go)
build: rust legacy-componentizejs go

# Build Rust components
rust:
	(cd activity/github/impl && cargo build --release)
	(cd activity/db/turso && cargo build --release)
	(cd activity/llm/openai && cargo build --release)
	(cd webhook/webhook-rs && cargo build --release)
	(cd workflow/stargazers/workflow-rs && cargo build --release)

# Direct JavaScript components are loaded by Obelisk and do not need a build.
js:
	@echo "JavaScript components do not need a build step"

# Build legacy ComponentizeJS components
legacy-componentizejs:
	./scripts/build-components-legacy-componentizejs.sh

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
test-e2e-js: rust js
	./scripts/test-e2e.sh ./obelisk-local-js-all.toml truncate
test-e2e-legacy-componentizejs: rust legacy-componentizejs
	./scripts/test-e2e.sh ./obelisk-local-legacy-componentizejs-all.toml truncate
test-e2e-go: go
	./scripts/test-e2e.sh ./obelisk-local-go-all.toml truncate
