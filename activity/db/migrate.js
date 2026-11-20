// JavaScript implementation of stargazers:db/migration.migrate
// A port of refinery's runner sharing its history table, so both implementations can migrate the same database.
import { checksum } from './checksum.js';
import { MIGRATIONS } from './migrations.js';
import { splitStatements } from './split-statements.js';
import { transaction } from './turso.js';

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

export default async function migrate(latestVersion) {
    const known = MIGRATIONS.map((m) => ({ ...m, checksum: checksum(m.name, m.version, m.sql) }));
    const latestKnown = Math.max(...known.map((m) => m.version));
    if (latestKnown !== latestVersion) throw `latest migration is ${latestKnown}, expected ${latestVersion}`;
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
