// Shared GitHub GraphQL client for the GitHub activities.
// In the cloud GITHUB_API_BASE_URL is the GitHub integration proxy, which authenticates requests itself.
export async function query(query, variables) {
    const token = process.env.GITHUB_TOKEN_STARGAZERS;
    const baseUrl = process.env.GITHUB_API_BASE_URL || 'https://api.github.com';
    const headers = {
        'Content-Type': 'application/json',
        'User-Agent': 'demo-stargazers',
    };
    if (token) {
        headers.Authorization = `Bearer ${token}`;
    }
    let response;
    try {
        response = await fetch(`${baseUrl}/graphql`, {
            method: 'POST',
            headers,
            body: JSON.stringify({ query, variables }),
        });
    } catch (e) {
        throw `cannot send the request - ${e.message}`;
    }
    if (response.status !== 200) throw `Unexpected status code: ${response.status}`;
    return await response.json();
}
