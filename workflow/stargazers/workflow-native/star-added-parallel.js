// Native JS implementation of stargazers:workflow/workflow.star-added-parallel
// Fetches account info and LLM settings concurrently using join sets.

import { accountInfoSubmit, accountInfoAwaitNext } from 'stargazers:github-obelisk-ext/account';
import { getSettingsJsonSubmit, getSettingsJsonAwaitNext } from 'stargazers:db-obelisk-ext/llm';
import { addStarGetDescription, updateUserDescription } from 'stargazers:db/user';
import { respond as llmRespond } from 'stargazers:llm/llm';

export default function star_added_parallel(login, repo) {
    const existingDescription = addStarGetDescription(login, repo);

    if (existingDescription !== null && existingDescription !== undefined) {
        console.log(`Description already exists for ${login} on ${repo}.`);
        return;
    }

    console.log(`No description for ${login} on ${repo}, generating in parallel...`);

    // Submit account-info and get-settings-json concurrently.
    const jsInfo = obelisk.createJoinSet({ name: `info_${login}` });
    accountInfoSubmit(jsInfo, login);

    const jsSettings = obelisk.createJoinSet({ name: `settings_${login}` });
    getSettingsJsonSubmit(jsSettings);

    // Await account info.
    const infoResult = accountInfoAwaitNext(jsInfo);

    // Await settings.
    const settingsResult = getSettingsJsonAwaitNext(jsSettings);

    console.log(`Got info and settings for ${login}, generating description...`);

    // Generate and persist the description sequentially.
    const description = llmRespond(infoResult, settingsResult);
    updateUserDescription(login, description);

    console.log(`Generated and saved description in parallel for ${login}`);
}
