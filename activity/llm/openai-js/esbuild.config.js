import esbuild from 'esbuild';
import { execSync } from 'child_process';

function getWitJson(witPath = './wit') {
    const cmd = `wasm-tools component wit --json ${witPath}`;
    const output = execSync(cmd, { encoding: 'utf8' });
    return JSON.parse(output);
}

function getRootImports(witJson) {
    const rootWorlds = witJson.worlds.filter(w => w.name === "root");
    if (rootWorlds.length !== 1) throw new Error("Expected exactly one `root` world.");

    const root = rootWorlds[0];
    const imports = root.imports || {};

    const result = [];

    for (const [_, imp] of Object.entries(imports)) {
        if (!imp.interface) continue;

        const ifcId = imp.interface.id;
        const ifc = witJson.interfaces[ifcId];
        const ifcName = ifc.name;
        const pkgId = ifc.package;
        const pkg = witJson.packages[pkgId];

        const [namespace, rest] = pkg.name.split(":");
        const [pkgName, version] = rest.split("@");

        const ifcFqn = `${namespace}:${pkgName}/${ifcName}` + (version ? `@${version}` : "");
        result.push(ifcFqn);
    }

    return result;
}

const witJson = getWitJson();
const external = getRootImports(witJson);
console.log("Excluding imports:", external);

esbuild.build({
    entryPoints: ['src/index.js'],
    bundle: true,
    platform: 'browser',
    outfile: 'bundle/index.bundled.js',
    format: 'esm',
    external,
})
