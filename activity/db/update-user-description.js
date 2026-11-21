// JavaScript implementation of stargazers:db/user.update-user-description
import { transaction, NEXT_UPDATED_AT } from './turso.js';

export default async function update_user_description(username, description) {
    const now = new Date().toISOString();
    await transaction([
        `INSERT INTO users (name, description, updated_at) VALUES (:login, :description, ${NEXT_UPDATED_AT}) ` +
        'ON CONFLICT(name) DO UPDATE SET description = excluded.description, updated_at = excluded.updated_at',
        { login: username, description, now },
    ]);
}
