// Applies src/activity/db/migrations to the Turso database at TURSO_LOCATION using TURSO_TOKEN.
// A port of refinery's runner sharing its history table, so the Rust activity can migrate the same database.
// Usage: node src/activity/db/migrate/migrate.mjs
import { readdirSync, readFileSync } from 'node:fs';
import { checksum } from './checksum.js';
import { splitStatements } from './split-statements.js';
import { transaction } from '../turso.js';

const TABLE = 'refinery_schema_history';

// Returns the migrations to apply, failing like refinery on divergent or missing ones.
export function pendingMigrations(applied, known) {
    for (const app of applied) {
        const migration = known.find((m) => m.version === app.version);
        if (!migration) throw `applied migration V${app.version}__${app.name} is missing`;
        if (migration.name !== app.name || migration.checksum !== app.checksum) {
            throw `applied migration V${app.version}__${app.name} is different than V${migration.version}__${migration.name}`;
        }
    }
    const current = applied.length > 0 ? applied[applied.length - 1].version : -1;
    const pending = [];
    for (const migration of [...known].sort((a, b) => a.version - b.version)) {
        if (applied.some((app) => app.version === migration.version)) continue;
        if (pending.some((m) => m.version === migration.version)) throw `repeated migration version ${migration.version}`;
        if (migration.version <= current) throw `migration V${migration.version}__${migration.name} was not applied`;
        pending.push(migration);
    }
    return pending;
}

const DIR = new URL('../migrations/', import.meta.url);

// The `V{version}__{name}.sql` files with their refinery checksums.
export function loadMigrations() {
    return readdirSync(DIR).map((file) => {
        const [, version, name] = file.match(/^V(\d+)__(\w+)\.sql$/) ?? [];
        if (!version) throw new Error(`unexpected migration file name ${file}`);
        const sql = readFileSync(new URL(file, DIR), 'utf8');
        return { version: Number(version), name, sql, checksum: checksum(name, Number(version), sql) };
    });
}

// Returns versions of the newly applied migrations.
export async function migrate() {
    const known = loadMigrations();
    await transaction([`CREATE TABLE IF NOT EXISTS ${TABLE}(version INT4 PRIMARY KEY, name VARCHAR(255), ` +
        'applied_on VARCHAR(255), checksum VARCHAR(255))']);
    const [result] = await transaction([`SELECT version, name, applied_on, checksum FROM ${TABLE} ORDER BY version ASC`]);
    const applied = result.rows.map(([version, name, , checksum]) =>
        ({ version: Number(version.value), name: name.value, checksum: checksum.value }));
    const versions = [];
    for (const migration of pendingMigrations(applied, known)) {
        // The migration and its history row commit together; a concurrent run fails on the primary key.
        await transaction(
            ...splitStatements(migration.sql).map((sql) => [sql]),
            [`INSERT INTO ${TABLE} (version, name, applied_on, checksum) VALUES (:version, :name, :applied_on, :checksum)`, {
                version: String(migration.version),
                name: migration.name,
                applied_on: new Date().toISOString(),
                checksum: migration.checksum,
            }],
        );
        versions.push(migration.version);
    }
    return versions;
}

if (import.meta.url === `file://${process.argv[1]}`) {
    try {
        const versions = await migrate();
        console.log(versions.length ? `Applied migrations ${versions.join(', ')}` : 'No pending migrations');
    } catch (e) {
        console.error(`Migration failed: ${e}`);
        process.exit(1);
    }
}
