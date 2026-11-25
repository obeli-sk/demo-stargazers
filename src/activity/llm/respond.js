// JavaScript implementation of stargazers:llm/llm.respond
// Calls an OpenAI-compatible chat completions endpoint.
export default async function respond(userPrompt, settingsString) {
    const baseUrl = baseUrlFromEnv();

    let settings;
    try {
        settings = JSON.parse(settingsString);
    } catch (e) {
        throw `'settings_json' must be parseable: ${e.message}`;
    }

    const messages = [
        ...(settings.messages || []),
        { role: 'user', content: userPrompt },
    ];

    const headers = { ...authHeaders(), 'Content-Type': 'application/json' };

    let response;
    try {
        response = await fetch(`${baseUrl}/v1/chat/completions`, {
            method: 'POST',
            headers,
            body: JSON.stringify({
                model: settings.model,
                messages,
                max_tokens: settings.max_tokens,
            }),
        });
    } catch (e) {
        throw `Network error: ${e.message}`;
    }

    if (!response.ok) {
        const text = await response.text();
        throw `API Error: ${response.status} - ${response.statusText}. Details: ${text}`;
    }

    const data = await response.json();

    if (data.choices && data.choices.length > 0 && data.choices[0].message) {
        return data.choices[0].message.content;
    }

    throw 'No response content from the LLM or choices array is malformed.';
}

export function baseUrlFromEnv() {
    const value = process.env['LLM_BASE_URL'];
    if (!value) throw 'LLM_BASE_URL is not defined';
    return value.replace(/\/+$/, '');
}

// The cloud LLM integration authenticates requests itself, so the key is optional.
export function authHeaders() {
    const apiKey = process.env['LLM_API_KEY'];
    return apiKey ? { 'Authorization': `Bearer ${apiKey}` } : {};
}
