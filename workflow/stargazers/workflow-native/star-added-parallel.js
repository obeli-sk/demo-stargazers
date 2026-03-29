// Native JS implementation of stargazers:workflow/workflow.star-added-parallel
// Fetches account info and LLM settings concurrently using join sets.

export default function star_added_parallel(login, repo) {
    const existingDescription = obelisk.call(
        'stargazers:db/user.add-star-get-description', [login, repo]);

    if (existingDescription !== null && existingDescription !== undefined) {
        console.log(`Description already exists for ${login} on ${repo}.`);
        return;
    }

    console.log(`No description for ${login} on ${repo}, generating in parallel...`);

    // Submit account-info and get-settings-json concurrently.
    const jsInfo = obelisk.createJoinSet({ name: `info_${login}` });
    const infoExecId = jsInfo.submit('stargazers:github/account.account-info', [login]);

    const jsSettings = obelisk.createJoinSet({ name: `settings_${login}` });
    const settingsExecId = jsSettings.submit('stargazers:db/llm.get-settings-json', []);

    // Await account info.
    jsInfo.joinNext();
    const infoResult = obelisk.getResult(infoExecId);
    if (infoResult.err !== undefined) throw infoResult.err;

    // Await settings.
    jsSettings.joinNext();
    const settingsResult = obelisk.getResult(settingsExecId);
    if (settingsResult.err !== undefined) throw settingsResult.err;

    console.log(`Got info and settings for ${login}, generating description...`);

    // Generate and persist the description sequentially.
    const description = obelisk.call('stargazers:llm/llm.respond',
        [infoResult.ok, settingsResult.ok]);
    obelisk.call('stargazers:db/user.update-user-description', [login, description]);

    console.log(`Generated and saved description in parallel for ${login}`);
}
