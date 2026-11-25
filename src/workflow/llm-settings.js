// Shared by the star-added workflows.
import { listModels } from 'stargazers:llm/llm';

// Fills in the first model offered by the LLM endpoint when the settings do not name one.
export function withModel(settingsJson) {
    let settings;
    try {
        settings = JSON.parse(settingsJson);
    } catch (e) {
        throw `LLM settings must be JSON: ${e.message}`;
    }
    if (settings.model) return settingsJson;
    const [model] = listModels();
    if (!model) throw 'The LLM endpoint offers no models';
    console.log(`LLM settings name no model, using ${model}`);
    return JSON.stringify({ ...settings, model });
}
