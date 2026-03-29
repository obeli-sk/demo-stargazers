// Native JS implementation of stargazers:workflow/workflow.star-added

export default function star_added(login, repo) {
    // 1. Persist the star and check whether a description already exists.
    // add-star-get-description: func(login: string, repo: string) -> result<option<string>, string>
    // obelisk.call unwraps the result: returns the ok value or throws the err string.
    const existingDescription = obelisk.call(
        'stargazers:db/user.add-star-get-description', [login, repo]);

    if (existingDescription !== null && existingDescription !== undefined) {
        console.log(`Description already exists for ${login} on ${repo}.`);
        return;
    }

    console.log(`No description for ${login} on ${repo}, generating...`);

    // 2. Fetch the account info from GitHub.
    const info = obelisk.call('stargazers:github/account.account-info', [login]);

    // 3. Fetch the LLM prompt settings from the database.
    const settingsJson = obelisk.call('stargazers:db/llm.get-settings-json', []);

    // 4. Generate the user's description using the LLM.
    const description = obelisk.call('stargazers:llm/llm.respond', [info, settingsJson]);

    // 5. Persist the generated description.
    obelisk.call('stargazers:db/user.update-user-description', [login, description]);

    console.log(`Generated and saved description for ${login}`);
}
