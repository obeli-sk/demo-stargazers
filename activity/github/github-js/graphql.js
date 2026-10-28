// Shared GitHub GraphQL client for the github-js activities.
export async function query(query, variables) {
    const token = process.env.GITHUB_TOKEN_STARGAZERS;
    if (!token) throw 'GITHUB_TOKEN_STARGAZERS must be passed as environment variable';
    const baseUrl = process.env.GITHUB_API_BASE_URL || 'https://api.github.com';
    let response;
    try {
        response = await fetch(`${baseUrl}/graphql`, {
            method: 'POST',
            headers: {
                'Authorization': `Bearer ${token}`,
                'Content-Type': 'application/json',
                'User-Agent': 'demo-stargazers',
            },
            body: JSON.stringify({ query, variables }),
        });
    } catch (e) {
        throw `cannot send the request - ${e.message}`;
    }
    if (response.status !== 200) throw `Unexpected status code: ${response.status}`;
    return await response.json();
}
