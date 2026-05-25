// Native JS implementation of stargazers:workflow/workflow.backfill-parallel
// Pages through all stargazers and submits star-added-parallel for each login
// concurrently within each page.

import { listStargazers } from 'stargazers:github/account';
import { starAddedParallelSubmit } from 'stargazers:workflow-obelisk-ext/workflow';

export default function backfill_parallel(repo) {
    console.log(`Starting parallel backfill for ${repo}...`);
    const pageSize = 5;
    let cursor = null;

    while (true) {
        const resp = listStargazers(repo, pageSize, cursor);

        if (!resp) {
            console.log('No more stargazers found.');
            break;
        }

        const gotWholePage = resp.logins.length === pageSize;
        console.log(`Found ${resp.logins.length} stargazers.`);

        // Submit all logins in this page concurrently.
        const joinSets = [];
        for (const login of resp.logins) {
            console.log(`Submitting task for ${login}...`);
            const js = obelisk.createJoinSet({ name: login });
            starAddedParallelSubmit(js, login, repo);
            joinSets.push(js);
        }

        // Close each join set to wait for that child execution to complete.
        for (const js of joinSets) {
            js.close();
        }

        if (!gotWholePage) {
            console.log('Reached last page.');
            break;
        }

        cursor = resp.cursor;
        console.log(`Moving to next page with cursor ${cursor}`);
    }

    console.log(`Parallel backfill for ${repo} completed.`);
}
