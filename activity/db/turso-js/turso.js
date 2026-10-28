// Shared Turso (libSQL Hrana over HTTP) client for the turso-js activities.

// Runs the statements atomically in one stateless request and returns each statement's result.
// Several statements run in an immediate transaction, each one only if the previous succeeded.
// Each statement is `[sql, namedArgs]`; string args are sent as text.
export async function transaction(...statements) {
    const token = process.env.TURSO_TOKEN;
    if (!token) throw 'TURSO_TOKEN must be set as an environment variable';
    const location = process.env.TURSO_LOCATION;
    if (!location) throw 'TURSO_LOCATION must be set as an environment variable';
    const ok = (step) => ({ type: 'ok', step });
    const explicit = statements.length > 1;
    const steps = explicit ? [{ stmt: { sql: 'BEGIN IMMEDIATE' } }] : [];
    const first = steps.length;
    for (const [sql, args = {}] of statements) {
        steps.push({
            condition: steps.length > 0 ? ok(steps.length - 1) : undefined,
            stmt: {
                sql,
                named_args: Object.entries(args).map(([name, value]) => ({ name, value: { type: 'text', value } })),
            },
        });
    }
    if (explicit) {
        steps.push({ condition: ok(steps.length - 1), stmt: { sql: 'COMMIT' } });
        steps.push({ condition: { type: 'not', cond: ok(steps.length - 1) }, stmt: { sql: 'ROLLBACK' } });
    }
    const body = JSON.stringify({ requests: [{ type: 'batch', batch: { steps } }, { type: 'close' }] });
    // Turso rejects chunked request bodies.
    const response = await fetch(`${location}/v2/pipeline`, {
        method: 'POST',
        headers: { 'Authorization': `Bearer ${token}`, 'Content-Type': 'application/json', 'Content-Length': String(new TextEncoder().encode(body).length) },
        body,
    });
    const text = await response.text();
    if (response.status !== 200) throw `Unexpected status code ${response.status}: ${text}`;
    const [batch, close] = JSON.parse(text).results;
    if (batch?.type !== 'ok' || batch.response.type !== 'batch') throw `Got batch error ${JSON.stringify(batch?.error ?? batch)}`;
    if (close?.type !== 'ok') throw `Got close error ${JSON.stringify(close?.error ?? close)}`;
    const { step_results: results, step_errors: errors } = batch.response.result;
    const error = errors.find((error) => error);
    if (error) throw `Got statement error ${JSON.stringify(error)}`;
    // Without errors, a missing COMMIT result means the transaction did not commit.
    if (explicit && !results[steps.length - 2]) throw 'Transaction was not committed';
    return results.slice(first, first + statements.length);
}

// The first cell of the first row, or a null value when there are no rows.
export function firstValue(result) {
    const row = result.rows[0];
    if (!row) return { type: 'null' };
    if (row.length === 0) throw 'No cell in the first row';
    return row[0];
}

// A text or null cell as a string or null.
export function optionalText(value) {
    if (value.type === 'text') return value.value;
    if (value.type === 'null') return null;
    throw `unexpected data type, expected Text or Null, got ${JSON.stringify(value)}`;
}
