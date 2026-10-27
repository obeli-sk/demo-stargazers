// JavaScript implementation of stargazers:db/user.list-stargazers
import { pipeline, optionalText } from './turso.js';

export default async function list_stargazers(last, repo, ordering) {
    const order = ordering === 'descending' ? 'DESC' : '';
    const where = repo === null ? '' : 'WHERE s.repo_name = :repo';
    const [result] = await pipeline([
        'SELECT u.name AS login, u.description, s.repo_name AS repo FROM users u ' +
        `INNER JOIN stars s ON u.name = s.user_name ${where} ` +
        `ORDER BY u.updated_at ${order}, s.rowid ${order} LIMIT ${last}`,
        repo === null ? {} : { repo },
    ]);
    const cols = result.cols.map((col) => col.name).join(',');
    if (cols !== 'login,description,repo') throw `wrong cols returned: ${cols}`;
    return result.rows.map(([login, description, repo]) => {
        const required = (value) => {
            const text = optionalText(value);
            if (text === null) throw 'mandatory value is missing';
            return text;
        };
        return { login: required(login), description: optionalText(description), repo: required(repo) };
    });
}
