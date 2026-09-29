# Build all components
build: rust js

# Build Rust components
rust:
	(cd activity/github/impl && env -u CARGO_TARGET_DIR cargo build --release)
	(cd activity/db/turso && env -u CARGO_TARGET_DIR cargo build --release)
	(cd activity/llm/openai && env -u CARGO_TARGET_DIR cargo build --release)
	(cd webhook/webhook-rs && env -u CARGO_TARGET_DIR cargo build --release)
	(cd workflow/stargazers/workflow-rs && env -u CARGO_TARGET_DIR cargo build --release)

# Direct JavaScript components are loaded by Obelisk and do not need a build.
js:
	@echo "JavaScript components do not need a build step"

serve:
	obelisk server run --app-config ./app.toml --deployment ./deployment-rs.toml

serve-js: rust
	obelisk server run --app-config ./app.toml --deployment ./deployment-js.toml

test-unit:
	./scripts/test-unit.sh
test-integration:
	./scripts/test-integration.sh
test-e2e: rust
	./scripts/test-e2e.sh ./deployment-rs.toml
test-e2e-js: rust js
	./scripts/test-e2e.sh ./deployment-js.toml
