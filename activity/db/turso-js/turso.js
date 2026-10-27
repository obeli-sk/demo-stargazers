// Shared Turso (libSQL Hrana over HTTP) client for the turso-js activities.

// Runs the statements in one pipeline and returns each statement's result.
// Each statement is `[sql, namedArgs]`; string args are sent as text.
export async function pipeline(...statements) {
    const token = process.env.TURSO_TOKEN;
    if (!token) throw 'TURSO_TOKEN must be set as an environment variable';
    const location = process.env.TURSO_LOCATION;
    if (!location) throw 'TURSO_LOCATION must be set as an environment variable';
    const requests = statements.map(([sql, args = {}]) => ({
        type: 'execute',
        stmt: {
            sql,
            named_args: Object.entries(args).map(([name, value]) => ({ name, value: { type: 'text', value } })),
        },
    }));
    requests.push({ type: 'close' });
    const body = JSON.stringify({ requests });
    // Turso rejects chunked request bodies.
    const response = await fetch(`${location}/v2/pipeline`, {
        method: 'POST',
        headers: { 'Authorization': `Bearer ${token}`, 'Content-Type': 'application/json', 'Content-Length': String(new TextEncoder().encode(body).length) },
        body,
    });
    const text = await response.text();
    if (response.status !== 200) throw `Unexpected status code ${response.status}: ${text}`;
    const results = JSON.parse(text).results.map((result) => {
        if (result.type !== 'ok') throw `Got response result error ${JSON.stringify(result.error)}`;
        return result.response;
    });
    return results.slice(0, -1).map((response) => {
        if (response.type !== 'execute' || !response.result) throw `unexpected response ${JSON.stringify(response)}`;
        return response.result;
    });
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
