// Native JS implementation of stargazers:workflow/workflow.star-added

import { accountInfo } from 'stargazers:github/account';
import { getSettingsJson } from 'stargazers:db/llm';
import { addStarGetDescription, updateUserDescription } from 'stargazers:db/user';
import { respond as llmRespond } from 'stargazers:llm/llm';

export default function star_added(login, repo) {
    // 1. Persist the star and check whether a description already exists.
    // add-star-get-description: func(login: string, repo: string) -> result<option<string>, string>
    const existingDescription = addStarGetDescription(login, repo);

    if (existingDescription !== null && existingDescription !== undefined) {
        console.log(`Description already exists for ${login} on ${repo}.`);
        return;
    }

    console.log(`No description for ${login} on ${repo}, generating...`);

    // 2. Fetch the account info from GitHub.
    const info = accountInfo(login);

    // 3. Fetch the LLM prompt settings from the database.
    const settingsJson = getSettingsJson();

    // 4. Generate the user's description using the LLM.
    const description = llmRespond(info, settingsJson);

    // 5. Persist the generated description.
    updateUserDescription(login, description);

    console.log(`Generated and saved description for ${login}`);
}
