import assert from 'node:assert/strict';
import { test } from 'node:test';
import { checksum } from '../checksum.js';
import { loadMigrations, pendingMigrations } from '../migrate.mjs';

test('checksum matches refinery', () => {
    // Same vector as the Rust `checksum_should_match_the_js_port` test.
    assert.equal(checksum('vector', 7, "SELECT 'ž';"), '16225368461553448942');
});

test('migration files load in a valid order', () => {
    assert.ok(pendingMigrations([], loadMigrations()).length > 0);
});

const m = (version, name, checksum = `${name}-sum`) => ({ version, name, checksum });

test('pending migrations follow the applied ones', () => {
    assert.deepEqual(pendingMigrations([m(1, 'a')], [m(2, 'b'), m(1, 'a'), m(3, 'c')]), [m(2, 'b'), m(3, 'c')]);
    assert.deepEqual(pendingMigrations([], [m(1, 'a')]), [m(1, 'a')]);
});

test('divergent, missing and repeated migrations are rejected', () => {
    assert.throws(() => pendingMigrations([m(1, 'a', 'other')], [m(1, 'a')]));
    assert.throws(() => pendingMigrations([m(1, 'a')], [m(1, 'renamed', 'a-sum')]));
    assert.throws(() => pendingMigrations([m(2, 'b')], [m(1, 'a'), m(2, 'b')]));
    assert.throws(() => pendingMigrations([m(1, 'a')], [m(2, 'b')]));
    assert.throws(() => pendingMigrations([], [m(1, 'a'), m(1, 'b')]));
});
