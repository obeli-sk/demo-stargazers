// JavaScript implementation of stargazers:db/user.remove-star
import { transaction } from './turso.js';

export default async function remove_star(login, repo) {
    await transaction(
        ['DELETE FROM stars WHERE repo_name = :repo AND user_name = :login', { login, repo }],
        ['DELETE FROM users WHERE name = :login AND NOT EXISTS (SELECT 1 FROM stars WHERE user_name = :login)', { login }],
    );
}
