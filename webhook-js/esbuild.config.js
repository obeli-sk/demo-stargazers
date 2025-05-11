import esbuild from 'esbuild';

esbuild.build({
    entryPoints: ['src/index.js'],
    bundle: true,
    platform: 'browser',
    outfile: 'bundle/index.bundled.js',
    format: 'esm',
    external: ['wasi:cli/environment@0.2.3',
        'wasi:http/types@0.2.3',
        'obelisk:log/log@1.0.0',
        'stargazers:db/user',
        'stargazers:workflow-obelisk-ext/workflow'
    ],
})
