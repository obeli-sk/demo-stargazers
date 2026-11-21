// JavaScript implementation of stargazers:llm/llm.list-models
// Returns the IDs of models offered by the OpenAI-compatible endpoint, in its order.
import { authHeaders, baseUrlFromEnv } from './respond.js';

export default async function listModels() {
    let response;
    try {
        response = await fetch(`${baseUrlFromEnv()}/v1/models`, { headers: authHeaders() });
    } catch (e) {
        throw `Network error: ${e.message}`;
    }
    if (!response.ok) {
        const text = await response.text();
        throw `API Error: ${response.status} - ${response.statusText}. Details: ${text}`;
    }
    const { data } = await response.json();
    if (!Array.isArray(data)) throw 'Model list has no `data` array';
    return data.map((model) => model.id);
}
