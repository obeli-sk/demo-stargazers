// Native JS implementation of stargazers:workflow/workflow.star-removed

import { removeStar } from 'stargazers:db/user';

export default function star_removed(login, repo) {
    // remove-star: func(login: string, repo: string) -> result<_, string>
    removeStar(login, repo);
    console.log(`Removed star for ${login} on ${repo}.`);
}
