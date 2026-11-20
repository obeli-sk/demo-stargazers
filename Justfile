# JavaScript components are loaded by Obelisk directly, without a build step.
serve:
	obelisk server run --dev --app-config ./app.toml --deployment ./deployment.toml

verify:
	obelisk server verify --skip-db --allow-unavailable-runtime-config --app-config ./app.toml --deployment ./deployment.toml

test-unit:
	node --test activity/db/*.test.mjs

test-e2e:
	./scripts/test-e2e.sh ./deployment.toml
