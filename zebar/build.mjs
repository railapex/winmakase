import { createHash } from 'node:crypto';
import { mkdir, readFile, rename, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { build } from 'esbuild';

const root = path.dirname(fileURLToPath(import.meta.url));
const checkOnly = process.argv.includes('--check');
const licensesOnly = process.argv.includes('--licenses');

const html = `<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <link rel="stylesheet" href="./styles.css" />
    <script defer src="./assets/bar.js"></script>
  </head>
  <body>
    <div id="root"></div>
  </body>
</html>
`;

async function compile() {
  const result = await build({
    absWorkingDir: root,
    bundle: true,
    define: { 'process.env.NODE_ENV': '"production"' },
    entryPoints: ['src/bar/main.tsx'],
    format: 'iife',
    jsx: 'automatic',
    legalComments: 'none',
    minify: true,
    outfile: 'assets/bar.js',
    platform: 'browser',
    sourcemap: false,
    target: ['chrome120'],
    treeShaking: true,
    write: false,
  });

  const javascript = result.outputFiles.find(file => file.path.endsWith('.js'));
  const css = result.outputFiles.find(file => file.path.endsWith('.css'));
  if (!javascript || !css) throw new Error('Expected JavaScript and CSS outputs');
  return {
    'assets/bar.js': javascript.contents,
    'bar.html': Buffer.from(html),
    'styles.css': css.contents,
  };
}

function digest(value) {
  return createHash('sha256').update(value).digest('hex');
}

async function writeAtomic(relativePath, contents) {
  const destination = path.join(root, relativePath);
  await mkdir(path.dirname(destination), { recursive: true });
  const temporary = `${destination}.tmp`;
  await writeFile(temporary, contents);
  await rename(temporary, destination);
}

async function assertGenerated(outputs) {
  for (const [relativePath, expected] of Object.entries(outputs)) {
    const actual = await readFile(path.join(root, relativePath));
    if (!actual.equals(expected)) {
      throw new Error(`${relativePath} is stale; run npm run build`);
    }
  }
}

function scanRuntimeAssets(outputs) {
  const findings = [];
  for (const [relativePath, contents] of Object.entries(outputs)) {
    const text = Buffer.from(contents).toString('utf8');
    const patterns = [
      /<(?:img|link|script)\b[^>]*(?:href|src)=["'](?:https?:)?\/\//gi,
      /@import\s+(?:url\()?\s*["']?(?:https?:)?\/\//gi,
      /url\(\s*["']?(?:https?:)?\/\//gi,
      /(?:fetch|import)\(\s*["'](?:https?:)?\/\//gi,
    ];
    const references = patterns.flatMap(pattern => text.match(pattern) ?? []);
    if (references.length) {
      findings.push(`${relativePath}: ${references.join(', ')}`);
    }
  }
  if (findings.length) {
    throw new Error(`Remote runtime asset references found:\n${findings.join('\n')}`);
  }
}

async function validateRuntimeConfiguration() {
  const runtimePath = path.join(root, 'assets/runtime-config.json');
  const zpackPath = path.join(root, 'zpack.json');
  const runtimeText = await readFile(runtimePath, 'utf8');
  const zpackText = await readFile(zpackPath, 'utf8');
  const hardcodedUserPath = /[A-Za-z]:[\\/]Users[\\/][^\\/"']+/i;
  if (hardcodedUserPath.test(runtimeText) || hardcodedUserPath.test(zpackText)) {
    throw new Error('Runtime configuration contains a hardcoded Windows user path');
  }

  const runtime = JSON.parse(runtimeText);
  const zpack = JSON.parse(zpackText);
  const program = runtime.launcher?.program;
  if (typeof program !== 'string') {
    throw new Error('runtime-config launcher.program must be a string');
  }
  const allowedPrograms = zpack.widgets.flatMap(widget =>
    (widget.privileges?.shellCommands ?? []).map(command => command.program),
  );
  if (program && !allowedPrograms.includes(program)) {
    throw new Error('Configured launcher is missing its exact Zebar shell privilege');
  }
  if (!program && allowedPrograms.length) {
    throw new Error('Zebar shell privileges must be empty when no launcher is configured');
  }
}

function packageName(packagePath) {
  const marker = 'node_modules/';
  const index = packagePath.lastIndexOf(marker);
  return packagePath.slice(index + marker.length);
}

async function noticeContents() {
  const lock = JSON.parse(await readFile(path.join(root, 'package-lock.json'), 'utf8'));
  const packages = Object.entries(lock.packages)
    .filter(([packagePath]) => packagePath)
    .map(([packagePath, metadata]) => ({
      name: packageName(packagePath),
      version: metadata.version,
      license: metadata.license ?? 'UNKNOWN',
      resolved: metadata.resolved ?? '',
      integrity: metadata.integrity ?? '',
      scope: metadata.dev ? 'build' : 'runtime',
    }))
    .sort((left, right) =>
      `${left.scope}:${left.name}`.localeCompare(`${right.scope}:${right.name}`),
    );

  const rows = packages.map(
    item =>
      `| ${item.name} | ${item.version} | ${item.scope} | ${item.license} | ${item.resolved} | ${item.integrity} |`,
  );
  const notice = `# Third-party notices

Generated from \`package-lock.json\` by \`node build.mjs --licenses\`. Runtime dependencies are bundled into \`assets/bar.js\`; build dependencies are used only to produce and type-check the committed assets. Exact source archives and integrity values are listed below. License texts remain in each archive and installed package. Zebar 3.3.1 is GPL-3.0-only; its corresponding source is the exact archive listed here and the upstream v3.3.1 tag.

| Package | Version | Use | License | Source archive | npm integrity |
|---|---:|---|---|---|---|
${rows.join('\n')}
`;
  return Buffer.from(notice);
}

if (licensesOnly) {
  await writeAtomic('THIRD-PARTY-NOTICES.md', await noticeContents());
  console.log('Updated THIRD-PARTY-NOTICES.md');
} else {
  await validateRuntimeConfiguration();
  const first = await compile();
  scanRuntimeAssets(first);
  const notices = await noticeContents();

  if (checkOnly) {
    const second = await compile();
    for (const relativePath of Object.keys(first)) {
      if (digest(first[relativePath]) !== digest(second[relativePath])) {
        throw new Error(`${relativePath} is not deterministic`);
      }
    }
    await assertGenerated({ ...first, 'THIRD-PARTY-NOTICES.md': notices });
    console.log('Generated assets are deterministic, current, and remote-asset free');
  } else {
    for (const [relativePath, contents] of Object.entries(first)) {
      await writeAtomic(relativePath, contents);
    }
    await writeAtomic('THIRD-PARTY-NOTICES.md', notices);
    console.log('Built bar.html, styles.css, assets/bar.js, and notices');
  }
}
