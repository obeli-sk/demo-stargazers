// Native JS implementation of stargazers:workflow/workflow.backfill
// Pages through all stargazers of a repo and calls star-added for each.

export default function backfill(repo) {
    console.log(`Starting backfill for ${repo}...`);
    const pageSize = 5;
    let cursor = null;

    while (true) {
        // list-stargazers: func(repo: string, page-size: u8, cursor: option<string>)
        //                  -> result<option<stargazers>, string>
        // Returns null (option None) when there are no more pages.
        const resp = obelisk.call(
            'stargazers:github/account.list-stargazers', [repo, pageSize, cursor]);

        if (!resp) {
            console.log('No more stargazers found.');
            break;
        }

        const gotWholePage = resp.logins.length === pageSize;
        console.log(`Found ${resp.logins.length} stargazers (page size ${pageSize}).`);

        for (const login of resp.logins) {
            console.log(`Processing ${login}...`);
            // star-added: func(login: string, repo: string) -> result<_, string>
            obelisk.call('stargazers:workflow/workflow.star-added', [login, repo]);
        }

        if (!gotWholePage) {
            console.log('Reached last page.');
            break;
        }

        cursor = resp.cursor;
        console.log(`Moving to next page with cursor ${cursor}`);
    }

    console.log(`Backfill for ${repo} completed.`);
}
