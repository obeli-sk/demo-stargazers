import esbuild from 'esbuild';

esbuild.build({
    entryPoints: ['src/activity.js'],
    bundle: true,
    platform: 'browser',
    outfile: 'bundle/activity.bundled.js',
    format: 'esm',
    external: ['wasi:cli/environment@0.2.3', 'obelisk:log/log@1.0.0'],
})
