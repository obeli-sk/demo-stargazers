// Lists the latest stargazers as HTML, or as JSON when requested with `Accept: application/json`.
// Served on the `internal` HTTP server: `/` for all repositories, `/:owner/:repo` for one.

import { listStargazers } from 'stargazers:db/user';

const LIMIT = 50;

const escape = (text) => text.replace(/[&<>"']/g, (c) =>
    ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c]);

export default function handle(request) {
    const owner = process.env['owner'];
    const repo = owner ? `${owner}/${process.env['repo']}` : null;
    let stargazers;
    try {
        stargazers = listStargazers(LIMIT, repo, 'descending');
    } catch (e) {
        console.error(`Cannot list stargazers: ${e}`);
        return new Response('Cannot list stargazers', { status: 500 });
    }
    if ((request.headers.get('accept') || '').includes('application/json')) {
        return Response.json(stargazers);
    }
    return new Response(page(repo, stargazers), {
        headers: { 'content-type': 'text/html; charset=utf-8' },
    });
}

function card({ login, repo, description }) {
    const user = encodeURIComponent(login);
    return `<li class="card">
  <img src="https://github.com/${user}.png?size=96" alt="" width="48" height="48" loading="lazy">
  <div>
    <div class="who"><a href="https://github.com/${user}">${escape(login)}</a>
      <a class="repo" href="/${repo.split('/').map(encodeURIComponent).join('/')}">${escape(repo)}</a></div>
    ${description === null
        ? '<p class="pending">Description is being generated&hellip;</p>'
        : `<p>${escape(description)}</p>`}
  </div>
</li>`;
}

function page(repo, stargazers) {
    const title = repo ? `Stargazers of ${repo}` : 'Latest stargazers';
    const back = repo
        ? `<a href="/">All repositories</a> &middot; <a href="https://github.com/${escape(repo)}">${escape(repo)} on GitHub</a>`
        : 'Most recently updated, across all repositories';
    const body = stargazers.length === 0
        ? '<p class="empty">No stargazers yet. Star the repository to see the webhook in action.</p>'
        : `<ul>${stargazers.map(card).join('')}</ul>`;
    return `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>${escape(title)}</title>
<style>
  :root { --bg: #f6f7f9; --card: #fff; --text: #1d2129; --muted: #626a75; --accent: #0b63ce; --border: #e1e4e8; }
  @media (prefers-color-scheme: dark) {
    :root { --bg: #111418; --card: #1a1f25; --text: #e6e8eb; --muted: #9aa3ad; --accent: #6aa9ff; --border: #2a3038; }
  }
  body { margin: 0; background: var(--bg); color: var(--text);
         font: 15px/1.5 system-ui, -apple-system, "Segoe UI", sans-serif; }
  main { max-width: 760px; margin: 0 auto; padding: 32px 16px; }
  h1 { margin: 0 0 4px; font-size: 24px; }
  .sub { margin: 0 0 24px; color: var(--muted); }
  a { color: var(--accent); text-decoration: none; }
  a:hover { text-decoration: underline; }
  ul { list-style: none; margin: 0; padding: 0; display: grid; gap: 12px; }
  .card { display: flex; gap: 14px; padding: 16px; background: var(--card);
          border: 1px solid var(--border); border-radius: 10px; }
  .card img { border-radius: 50%; flex: none; }
  .who { display: flex; flex-wrap: wrap; align-items: baseline; gap: 4px 10px; font-weight: 600; }
  .repo { font-weight: 400; font-size: 13px; color: var(--muted); }
  .card p { margin: 6px 0 0; overflow-wrap: anywhere; }
  .pending, .empty { color: var(--muted); font-style: italic; }
</style>
</head>
<body>
<main>
  <h1>${escape(title)}</h1>
  <p class="sub">${back}</p>
  ${body}
</main>
</body>
</html>
`;
}
