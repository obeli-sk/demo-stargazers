// Native JS implementation of the stargazers webhook endpoint.
//
// GitHub webhook signature verification uses crypto.subtle (HMAC-SHA256).
// Set the environment variable GITHUB_WEBHOOK_SECRET to the webhook secret,
// or set GITHUB_WEBHOOK_INSECURE=true to skip verification (development only).

const HTTP_HEADER_SIGNATURE = 'X-Hub-Signature-256';
const ENV_GITHUB_WEBHOOK_INSECURE = 'GITHUB_WEBHOOK_INSECURE';
const ENV_GITHUB_WEBHOOK_SECRET = 'GITHUB_WEBHOOK_SECRET';

// WIT enum ordering values (serialised as plain strings)
const ORDERING_ASCENDING = 'ascending';
const ORDERING_DESCENDING = 'descending';

// Convert hex string to Uint8Array.
function hexToUint8Array(hex) {
    if (hex.length % 2 !== 0) throw new Error('Invalid hex string');
    const bytes = new Uint8Array(hex.length / 2);
    for (let i = 0; i < hex.length; i += 2) {
        bytes[i / 2] = parseInt(hex.substring(i, i + 2), 16);
    }
    return bytes;
}

// Verify GitHub X-Hub-Signature-256 header using crypto.subtle (constant-time).
async function verifySignature(secret, bodyText, signatureHeader) {
    if (!signatureHeader) {
        throw new Error('Missing X-Hub-Signature-256 header');
    }
    const PREFIX = 'sha256=';
    if (!signatureHeader.startsWith(PREFIX)) {
        throw new Error(`X-Hub-Signature-256 must start with '${PREFIX}'`);
    }

    const signatureHex = signatureHeader.substring(PREFIX.length);
    let signatureBytes;
    try {
        signatureBytes = hexToUint8Array(signatureHex);
    } catch (e) {
        throw new Error(`X-Hub-Signature-256 must be hex-encoded: ${e.message}`);
    }

    const encoder = new TextEncoder();
    const keyData = encoder.encode(secret);
    const payload = encoder.encode(bodyText);

    const key = await crypto.subtle.importKey(
        'raw',
        keyData,
        { name: 'HMAC', hash: 'SHA-256' },
        false,
        ['verify'],
    );

    // crypto.subtle.verify performs constant-time comparison on the Rust side.
    const ok = await crypto.subtle.verify('HMAC', key, signatureBytes, payload);
    if (!ok) {
        throw new Error('Signature verification failed');
    }
    console.debug('Signature verified successfully.');
}

export default async function handle(request) {
    if (request.method === 'GET') {
        return handleGet(request);
    }
    if (request.method === 'POST') {
        return await handlePost(request);
    }
    return new Response('Method Not Allowed', { status: 405 });
}

// POST /  – receive a GitHub star event and schedule the appropriate workflow.
async function handlePost(request) {
    console.debug('Webhook POST received');

    let bodyText;
    try {
        bodyText = await request.text();
    } catch (e) {
        console.error(`Failed to read request body: ${e.message}`);
        return new Response('Failed to read body', { status: 400 });
    }

    const insecure = process.env[ENV_GITHUB_WEBHOOK_INSECURE] === 'true';
    if (insecure) {
        console.warn(`Not verifying the request because ${ENV_GITHUB_WEBHOOK_INSECURE} is set to 'true'!`);
    } else {
        const secret = process.env[ENV_GITHUB_WEBHOOK_SECRET];
        if (!secret) {
            console.error(`${ENV_GITHUB_WEBHOOK_SECRET} must be set as an environment variable`);
            return new Response('Not configured', { status: 500 });
        }
        const signatureHeader = request.headers.get(HTTP_HEADER_SIGNATURE);
        try {
            await verifySignature(secret, bodyText, signatureHeader);
        } catch (e) {
            console.error(`Signature verification failed: ${e.message}`);
            return new Response('Signature verification failed', { status: 403 });
        }
    }

    let event;
    try {
        event = JSON.parse(bodyText);
    } catch (e) {
        console.error(`Cannot deserialize JSON: ${e.message}`);
        return new Response('Invalid JSON payload', { status: 400 });
    }

    const repoFullName =
        `${event.repository?.owner?.login}/${event.repository?.name}`;
    const senderLogin = event.sender?.login;

    console.info(
        `Got event: Action: ${event.action}, Sender: ${senderLogin}, Repo: ${repoFullName}`);

    if (!senderLogin || !repoFullName.includes('/')) {
        console.error('Event data is malformed (missing sender.login or repository details).');
        return new Response('Malformed event data', { status: 400 });
    }

    let execId;
    if (event.action === 'created') {
        console.info(`Scheduling star_added for ${senderLogin} on ${repoFullName}`);
        execId = obelisk.executionIdGenerate();
        obelisk.schedule(execId, 'stargazers:workflow/workflow.star-added',
            [senderLogin, repoFullName]);
    } else if (event.action === 'deleted') {
        console.info(`Scheduling star_removed for ${senderLogin} on ${repoFullName}`);
        execId = obelisk.executionIdGenerate();
        obelisk.schedule(execId, 'stargazers:workflow/workflow.star-removed',
            [senderLogin, repoFullName]);
    } else {
        console.error(`Unknown action: ${event.action}`);
        return new Response(`Unknown action: ${event.action}`, { status: 400 });
    }

    console.info(`Workflow scheduled with execution ID: ${execId}`);
    return new Response('Webhook processed', {
        status: 200,
        headers: { 'execution-id': execId },
    });
}

// Parse query parameters from a URL's search string into a plain object.
function parseQueryParams(search) {
    const params = {};
    const query = search.startsWith('?') ? search.substring(1) : search;
    for (const part of query.split('&')) {
        if (!part) continue;
        const eqIdx = part.indexOf('=');
        if (eqIdx === -1) {
            params[decodeURIComponent(part)] = '';
        } else {
            params[decodeURIComponent(part.substring(0, eqIdx))] =
                decodeURIComponent(part.substring(eqIdx + 1));
        }
    }
    return params;
}

// GET /  – list recent stargazers from the database.
function handleGet(request) {
    console.info('GET request received');

    const url = new URL(request.url);
    const params = parseQueryParams(url.search);
    const MAX_LIMIT = 5;
    let limit = parseInt(params['limit'], 10);
    if (isNaN(limit) || limit <= 0) limit = MAX_LIMIT;
    limit = Math.min(limit, MAX_LIMIT);

    const repo = params['repo'] || null;
    const orderingParam = params['ordering'];
    const ordering = orderingParam === 'asc' ? ORDERING_ASCENDING : ORDERING_DESCENDING;

    console.info(`Listing stargazers: limit=${limit}, repo=${repo}, ordering=${ordering}`);

    // list-stargazers: func(last: u8, repo: option<string>, ordering: ordering)
    //                  -> result<list<stargazer>, string>
    const list = obelisk.call('stargazers:db/user.list-stargazers', [limit, repo, ordering]);
    return Response.json(list);
}
