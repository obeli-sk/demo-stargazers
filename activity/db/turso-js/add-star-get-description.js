// JavaScript implementation of stargazers:db/user.add-star-get-description
import { pipeline, firstValue, optionalText } from './turso.js';

export default async function add_star_get_description(login, repo) {
    const [star, user] = await pipeline(
        ['SELECT 1 FROM stars WHERE user_name = :login AND repo_name = :repo', { login, repo }],
        ['SELECT description FROM users WHERE name = :login', { login }],
    );
    const starValue = firstValue(star);
    if (starValue.type !== 'integer' && starValue.type !== 'null') {
        throw `unexpected data type, expected Integer or Null, got ${JSON.stringify(starValue)}`;
    }
    const description = optionalText(firstValue(user));
    // An existing star changes nothing, not even the user's `updated_at`.
    if (starValue.type === 'null') {
        const now = new Date().toISOString();
        await pipeline(
            ['INSERT INTO users (name, updated_at) VALUES (:login, :now) ON CONFLICT(name) DO UPDATE SET updated_at = :now', { login, now }],
            ['INSERT INTO repos (name) VALUES (:repo) ON CONFLICT DO NOTHING', { repo }],
            ['INSERT INTO stars (user_name, repo_name) VALUES (:login, :repo) ON CONFLICT DO NOTHING', { login, repo }],
        );
    }
    return description;
}
