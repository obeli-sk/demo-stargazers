# JavaScript components are loaded by Obelisk directly, without a build step.
serve:
	obelisk server run --dev --app-config ./app.toml --deployment ./deployment.toml

# Applies pending migrations to the database at TURSO_LOCATION.
migrate:
	node src/activity/db/migrate/migrate.mjs

verify:
	obelisk server verify --dev --skip-db --allow-unavailable-runtime-config --app-config ./app.toml --deployment ./deployment.toml

test-unit:
	node --test src/activity/db/migrate/test/*.test.mjs

test-e2e:
	./scripts/test-e2e.sh ./deployment.toml
