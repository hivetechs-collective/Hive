export type TemplateKind = 'node' | 'python' | 'rust' | 'empty';

export interface ScaffoldFile {
  path: string;      // relative path from project root
  content?: string;  // file contents
  isDir?: boolean;   // create directory only
}

export function getScaffoldFiles(template: TemplateKind, name: string): ScaffoldFile[] {
  const files: ScaffoldFile[] = [];
  const norm = (s: string) => s.replace(/[^a-zA-Z0-9_-]/g, '-');
  const safeName = norm(name || 'my-project');

  switch (template) {
    case 'node':
      files.push(
        { path: 'package.json', content: JSON.stringify({ name: safeName, version: '0.1.0', scripts: { start: 'node index.js' } }, null, 2) + '\n' },
        { path: 'index.js', content: "console.log('Hello from Node project');\n" },
        { path: '.gitignore', content: 'node_modules\n.DS_Store\n' },
        { path: 'README.md', content: `# ${name}\n\nCreated with Hive Consensus IDE.` }
      );
      break;
    case 'python':
      files.push(
        { path: 'main.py', content: "print('Hello from Python project')\n" },
        { path: '.gitignore', content: '__pycache__/\n.DS_Store\n' },
        { path: 'README.md', content: `# ${name}\n\nCreated with Hive Consensus IDE.` }
      );
      break;
    case 'rust':
      files.push(
        { path: 'src', isDir: true },
        { path: 'Cargo.toml', content: `[package]\nname = "${safeName}"\nversion = "0.1.0"\nedition = "2021"\n\n[dependencies]\n` },
        { path: 'src/main.rs', content: "fn main() {\n    println!(\"Hello from Rust project\");\n}\n" },
        { path: '.gitignore', content: 'target/\n.DS_Store\n' },
        { path: 'README.md', content: `# ${name}\n\nCreated with Hive Consensus IDE.` }
      );
      break;
    case 'empty':
    default:
      files.push(
        { path: 'README.md', content: `# ${name}\n\nCreated with Hive Consensus IDE.` }
      );
      break;
  }
  return files;
}

