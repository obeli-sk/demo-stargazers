import { Hono } from 'hono';

// WIT Imports
import { getEnvironment } from 'wasi:cli/environment@0.2.3';
import { debug as log_debug, info as log_info, warn as log_warn, error as log_error } from 'obelisk:log/log@1.0.0';
import { starAddedSchedule, starRemovedSchedule } from 'stargazers:workflow-obelisk-ext/workflow';
import { listStargazers } from 'stargazers:db/user';

const HTTP_HEADER_SIGNATURE = "X-Hub-Signature-256";
const ENV_GITHUB_WEBHOOK_INSECURE = "GITHUB_WEBHOOK_INSECURE";
const ENV_GITHUB_WEBHOOK_SECRET = "GITHUB_WEBHOOK_SECRET";
// Enums are strings.
// WIT: package stargazers:db/user enum ordering { ascending, descending, }
const OrderingAscending = 'ascending';
const OrderingDescending = 'descending';
// Variants are encoded with 'tag'.
// WIT: obelisk:types/time@1.1.0 variant schedule-at { now,.. }
const Now = { "tag": "now" };

// Helper function to convert ArrayBuffer to hex string
function arrayBufferToHex(buffer) {
    return [...new Uint8Array(buffer)]
        .map(b => b.toString(16).padStart(2, '0'))
        .join('');
}

// Helper function to convert hex string to Uint8Array
function hexToUint8Array(hexString) {
    if (hexString.length % 2 !== 0) {
        throw new Error("Invalid hex string");
    }
    const byteArray = new Uint8Array(hexString.length / 2);
    for (let i = 0; i < hexString.length; i += 2) {
        byteArray[i / 2] = parseInt(hexString.substring(i, i + 2), 16);
    }
    return byteArray;
}

/**
 * Verify a message using a shared secret and X-Hub-Signature-256 formatted hash.
 */
async function verifySignature(secret, payloadBody, sha256SignatureHeader) {
    if (!sha256SignatureHeader) {
        log_error("Missing X-Hub-Signature-256 header");
        throw new Error("Missing X-Hub-Signature-256 header");
    }

    const prefix = "sha256=";
    if (!sha256SignatureHeader.startsWith(prefix)) {
        log_error(`X-Hub-Signature-256 must start with ${prefix}`);
        throw new Error(`X-Hub-Signature-256 must start with ${prefix}`);
    }

    const signatureHex = sha256SignatureHeader.substring(prefix.length);
    let signatureBytes;
    try {
        signatureBytes = hexToUint8Array(signatureHex);
    } catch (e) {
        log_error(`X-Hub-Signature-256 must be hex-encoded: ${e.message}`);
        throw new Error(`X-Hub-Signature-256 must be hex-encoded: ${e.message}`);
    }

    const encoder = new TextEncoder();
    const keyData = encoder.encode(secret);
    const payloadData = typeof payloadBody === 'string' ? encoder.encode(payloadBody) : payloadBody; // Ensure payload is ArrayBuffer or TypedArray

    const key = await crypto.subtle.importKey(
        "raw",
        keyData,
        { name: "HMAC", hash: "SHA-256" },
        false,
        ["sign"]
    );

    const computedSignatureBuffer = await crypto.subtle.sign(
        "HMAC",
        key,
        payloadData
    );

    // Constant-time comparison is ideal, but for simplicity here:
    const computedSignatureHex = arrayBufferToHex(computedSignatureBuffer);
    if (computedSignatureHex !== signatureHex) {
        log_error(`Signature verification failed. Expected: ${signatureHex}, Got: ${computedSignatureHex}`);
        throw new Error("Signature verification failed");
    }
    log_debug("Signature verified successfully.");
}

const app = new Hono();

// POST endpoint for webhooks
app.post('/', async (c) => {
    log_debug("Webhook received");
    try {
        const sha256Signature = c.req.header(HTTP_HEADER_SIGNATURE);

        const rawBody = await c.req.arrayBuffer();
        const bodyText = new TextDecoder().decode(rawBody); // For parsing JSON later

        const env = new Map(getEnvironment());
        const insecure = env.get(ENV_GITHUB_WEBHOOK_INSECURE) === "true";

        if (insecure) {
            log_warn(`Not verifying the request because ${ENV_GITHUB_WEBHOOK_INSECURE} is set to 'true'!`);
        } else {
            const secret = env.get(ENV_GITHUB_WEBHOOK_SECRET);
            if (!secret) {
                log_error(`${ENV_GITHUB_WEBHOOK_SECRET} must be passed as environment variable`);
                return c.text("Not configured", 500);
            }
            try {
                await verifySignature(secret, rawBody, sha256Signature);
            } catch (err) {
                log_error(`Signature verification failed: ${err.message}`);
                return c.text('Signature verification failed', 403); // Forbidden
            }
        }

        let event;
        try {
            event = JSON.parse(bodyText); // Parse from the preserved text
        } catch (err) {
            log_error(`Cannot deserialize JSON: ${err.message}`);
            return c.text('Invalid JSON payload', 400); // Bad Request
        }

        log_info(`Got event: Action: ${event.action}, Sender: ${event.sender?.login}, Repo: ${event.repository?.owner?.login}/${event.repository?.name}`);

        // Assuming event.repository.owner.login and event.repository.name exist
        const repoFullName = `${event.repository?.owner?.login}/${event.repository?.name}`;
        const senderLogin = event.sender?.login;

        if (!senderLogin || !repoFullName.includes('/')) {
            log_error("Event data is malformed (missing sender.login or repository details).", senderLogin, repoFullName);
            return c.text('Malformed event data', 400);
        }

        let executionId;
        if (event.action === "created") {
            log_info(`Scheduling star_added for ${senderLogin} on ${repoFullName}`);
            executionId = starAddedSchedule(Now, senderLogin, repoFullName);
        } else if (event.action === "deleted") {
            log_info(`Scheduling star_removed for ${senderLogin} on ${repoFullName}`);
            executionId = starRemovedSchedule(Now, senderLogin, repoFullName);
        } else {
            log_error(`Unknown action: ${event.action}`);
            return c.text(`Unknown action: ${event.action}`, 400);
        }

        log_info(`Workflow scheduled with execution ID: ${executionId.id}`);
        c.header('execution-id', executionId.id);
        return c.text('Webhook processed', 200);

    } catch (err) {
        log_error(`Error handling webhook: ${err.message} ${err.stack}`);
        return c.text('Internal Server Error', 500);
    }
});

// GET endpoint to list stargazers
app.get('/', async (c) => {
    log_info("GET request received");
    try {
        const MAX_LIMIT = 5;
        let limit = parseInt(c.req.query('limit'), 10);
        if (isNaN(limit) || limit <= 0) {
            limit = MAX_LIMIT;
        }
        limit = Math.min(limit, MAX_LIMIT);

        const repo = c.req.query('repo') || undefined; // Pass undefined if not present

        let orderingParam = c.req.query('ordering');
        let ordering;
        if (orderingParam === "asc") {
            ordering = OrderingAscending;
        } else {
            ordering = OrderingDescending;
        }

        log_info(`Listing stargazers: limit=${limit}, repo=${repo}, ordering=${JSON.stringify(ordering)}`);

        // WIT: list-stargazers: func(last: u8, repo: option<string>, ordering: ordering) -> result<list<stargazer>, string>;
        const list = listStargazers(limit, repo, ordering);

        return c.json(list);

    } catch (err) {
        log_error(`Error handling GET request: ${err.message} ${err.stack}`);
        return c.text('Internal Server Error', 500);
    }
});


function serve(app) {
    self.addEventListener('fetch', (event) => {
        event.respondWith(app.fetch(event.request));
    });
}

serve(app);


