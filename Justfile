default: all

# Build all components (Rust, JS, Go)
all: rust js go

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

# List available tasks
help:
	just --list
