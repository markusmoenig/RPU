import {execFileSync} from 'node:child_process';
import {mkdtempSync, rmSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {fromHtml} from 'hast-util-from-html';
import {visit} from 'unist-util-visit';

type Node = {
  type: string;
  tagName?: string;
  properties?: Record<string, unknown>;
  children?: Node[];
  value?: string;
};

const repoRoot = path.resolve(__dirname, '../../..');
const treeSitterDir = path.join(repoRoot, 'tree-sitter-rpu');
const treeSitterBin = path.join(treeSitterDir, 'node_modules/.bin/tree-sitter');
const treeSitterConfig = path.join(repoRoot, 'docs/tree-sitter-rpu.config.json');

function extractText(node: Node): string {
  if (node.type === 'text') {
    return node.value ?? '';
  }
  return (node.children ?? []).map(extractText).join('');
}

function isRpuCodeBlock(node: Node): boolean {
  if (node.type !== 'element' || node.tagName !== 'pre') {
    return false;
  }
  const code = node.children?.find(
    (child) => child.type === 'element' && child.tagName === 'code',
  );
  if (!code) {
    return false;
  }
  const className = code.properties?.className;
  const classes = Array.isArray(className)
    ? className.map(String)
    : typeof className === 'string'
      ? [className]
      : [];
  return classes.includes('language-rpu');
}

function normalizeLineHtml(lineHtml: string): string {
  return lineHtml
    .replaceAll("<span style='color: #ff9e64'>", '<span class="rpu-token rpu-token-keyword">')
    .replaceAll("<span style='color: #7aa2f7'>", '<span class="rpu-token rpu-token-function">')
    .replaceAll("<span style='color: #7dcfff'>", '<span class="rpu-token rpu-token-property">')
    .replaceAll("<span style='color: #9ece6a'>", '<span class="rpu-token rpu-token-string">')
    .replaceAll("<span style='color: #e0af68'>", '<span class="rpu-token rpu-token-number">')
    .replaceAll("<span style='color: #d5def5'>", '<span class="rpu-token rpu-token-variable">')
    .replaceAll("<span style='color: #c4b5fd'>", '<span class="rpu-token rpu-token-builtin">')
    .replaceAll("<span style='color: #f5c981'>", '<span class="rpu-token rpu-token-parameter">')
    .replaceAll("<span style='color: #8b93aa'>", '<span class="rpu-token rpu-token-punctuation">')
    .replaceAll("<span style='color: #f7768e'>", '<span class="rpu-token rpu-token-special">')
    .replaceAll(
      "<span style='font-weight: bold;color: #89ddff'>",
      '<span class="rpu-token rpu-token-operator">',
    )
    .replaceAll(
      "<span style='font-style: italic;color: #6b7280'>",
      '<span class="rpu-token rpu-token-comment">',
    );
}

function highlightRpu(source: string): Node | null {
  const tempDir = mkdtempSync(path.join(tmpdir(), 'rpu-ts-'));
  const tempFile = path.join(tempDir, 'snippet.rpu');
  writeFileSync(tempFile, source, 'utf8');

  try {
    const output = execFileSync(
      treeSitterBin,
      [
        'highlight',
        '--html',
        '--config-path',
        treeSitterConfig,
        '--scope',
        'source.rpu',
        tempFile,
      ],
      {
        cwd: treeSitterDir,
        encoding: 'utf8',
      },
    );

    const match = output.match(/<table>[\s\S]*<\/table>/);
    if (!match) {
      return null;
    }
    const lines = [...match[0].matchAll(/<tr><td class=line-number>(.*?)<\/td><td class=line>([\s\S]*?)\n?<\/td><\/tr>/g)];
    if (lines.length === 0) {
      return null;
    }

    const lineHtml = lines
      .map(
        ([, lineNumber, lineContent]) =>
          `<span class="rpu-code-line"><span class="rpu-line-number" aria-hidden="true">${lineNumber}</span><span class="rpu-line-content">${normalizeLineHtml(lineContent || '&#8203;')}</span></span>`,
      )
      .join('');

    const fragment = fromHtml(
      `<div class="theme-code-block rpu-tree-sitter-highlight"><pre class="rpu-code-pre"><code class="language-rpu rpu-code">${lineHtml}</code></pre></div>`,
      {fragment: true},
    );
    return fragment.children[0] as Node;
  } finally {
    rmSync(tempDir, {recursive: true, force: true});
  }
}

export default function rehypeTreeSitterRpu() {
  return function transformer(tree: Node) {
    visit(tree as never, 'element', (node: Node, index: number | undefined, parent: Node | undefined) => {
      if (!parent || index === undefined || !isRpuCodeBlock(node)) {
        return;
      }

      const code = node.children?.find(
        (child) => child.type === 'element' && child.tagName === 'code',
      );
      if (!code) {
        return;
      }

      const highlighted = highlightRpu(extractText(code));
      if (!highlighted) {
        return;
      }

      parent.children ??= [];
      parent.children[index] = highlighted;
    });
  };
}
