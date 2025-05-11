import esbuild from 'esbuild';

esbuild.build({
    entryPoints: ['src/index.js'],
    bundle: true,
    platform: 'browser',
    outfile: 'bundle/index.bundled.js',
    format: 'esm',
    external: [
        'stargazers:workflow/workflow',
        'stargazers:workflow-obelisk-ext/workflow',
        'stargazers:github/account',
        'stargazers:github-obelisk-ext/account',
        'stargazers:db/llm',
        'stargazers:db-obelisk-ext/llm',
        'stargazers:db/user',
        'stargazers:db-obelisk-ext/user',
        'stargazers:llm/llm',
        'stargazers:llm-obelisk-ext/llm',
        'obelisk:workflow/workflow-support@1.1.0',
        'obelisk:log/log@1.0.0'
    ], // FIXME: generate from wit/world.wit
})
