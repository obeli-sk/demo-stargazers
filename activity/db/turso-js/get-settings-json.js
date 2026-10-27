// JavaScript implementation of stargazers:db/llm.get-settings-json
import { pipeline, firstValue } from './turso.js';

export default async function get_settings_json() {
    const [result] = await pipeline(['SELECT settings FROM llm WHERE id = 1']);
    const value = firstValue(result);
    if (value.type !== 'text') throw 'No text value in the first cell';
    return value.value;
}
