// JavaScript implementation of stargazers:db/user.update-user-description
import { pipeline } from './turso.js';

export default async function update_user_description(username, description) {
    const now = new Date().toISOString();
    await pipeline([
        'INSERT INTO users (name, description, updated_at) VALUES (:login, :description, :now) ' +
        'ON CONFLICT(name) DO UPDATE SET description = excluded.description, updated_at = :now',
        { login: username, description, now },
    ]);
}
