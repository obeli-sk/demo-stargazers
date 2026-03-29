// Native JS implementation of stargazers:workflow/workflow.star-removed

export default function star_removed(login, repo) {
    // remove-star: func(login: string, repo: string) -> result<_, string>
    obelisk.call('stargazers:db/user.remove-star', [login, repo]);
    console.log(`Removed star for ${login} on ${repo}.`);
}
