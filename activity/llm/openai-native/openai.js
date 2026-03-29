// Native JS implementation of stargazers:llm/llm.respond
export default async function respond(userPrompt, settingsString) {
    const apiKey = getEnv('OPENAI_API_KEY');
    const baseUrl = getEnv('OPENAI_API_BASE_URL');

    if (!apiKey) {
        throw 'OPENAI_API_KEY must be set as an environment variable';
    }

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

    let response;
    try {
        response = await fetch(`${baseUrl}/v1/chat/completions`, {
            method: 'POST',
            headers: {
                'Authorization': `Bearer ${apiKey}`,
                'Content-Type': 'application/json',
            },
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

    throw 'No response content from OpenAI or choices array is malformed.';
}

function getEnv(name) {
    const value = process.env[name];
    if (!value) throw `${name} is not defined`;
    return value;
}
