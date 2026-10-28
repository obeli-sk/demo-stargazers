import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';
import { splitStatements } from './split-statements.js';

const cases = JSON.parse(readFileSync(new URL('../split-statements.json', import.meta.url), 'utf8'));

for (const { name, sql, statements, error } of cases) {
    test(name, () => {
        if (error) {
            assert.throws(() => splitStatements(sql));
        } else {
            assert.deepEqual(splitStatements(sql), statements);
        }
    });
}
