/**
 * VS Code-style File Icon Mappings
 * Based on VS Code's Seti icon theme
 */

export interface IconMapping {
  icon: string;
  color?: string;
}

// Map file extensions to Codicon classes
export const fileIconMap: Record<string, IconMapping> = {
  // Programming Languages
  'js': { icon: 'symbol-namespace', color: '#f1dd35' },
  'jsx': { icon: 'symbol-namespace', color: '#f1dd35' },
  'ts': { icon: 'symbol-class', color: '#007acc' },
  'tsx': { icon: 'symbol-class', color: '#007acc' },
  'py': { icon: 'symbol-method', color: '#3776ab' },
  'java': { icon: 'symbol-interface', color: '#ea2d2e' },
  'cpp': { icon: 'symbol-misc', color: '#00599c' },
  'c': { icon: 'symbol-misc', color: '#555555' },
  'cs': { icon: 'symbol-class', color: '#178600' },
  'go': { icon: 'symbol-module', color: '#00add8' },
  'rs': { icon: 'symbol-keyword', color: '#ce4321' },
  'php': { icon: 'symbol-variable', color: '#4f5d95' },
  'rb': { icon: 'ruby', color: '#cc342d' },
  'swift': { icon: 'symbol-color', color: '#fa7343' },
  'kt': { icon: 'symbol-event', color: '#7f52ff' },
  'scala': { icon: 'symbol-ruler', color: '#dc322f' },
  'dart': { icon: 'symbol-color', color: '#00b4ab' },
  'lua': { icon: 'symbol-constant', color: '#000080' },
  'perl': { icon: 'symbol-regex', color: '#0298c3' },
  'r': { icon: 'graph-line', color: '#358a5b' },
  
  // Web Technologies
  'html': { icon: 'code', color: '#e34c26' },
  'htm': { icon: 'code', color: '#e34c26' },
  'css': { icon: 'symbol-color', color: '#1572b6' },
  'scss': { icon: 'symbol-color', color: '#cf649a' },
  'sass': { icon: 'symbol-color', color: '#cf649a' },
  'less': { icon: 'symbol-color', color: '#1d365d' },
  'vue': { icon: 'symbol-namespace', color: '#4fc08d' },
  'svelte': { icon: 'symbol-namespace', color: '#ff3e00' },
  
  // Data & Config
  'json': { icon: 'json', color: '#cbcb41' },
  'jsonc': { icon: 'json', color: '#cbcb41' },
  'yaml': { icon: 'settings-gear', color: '#cb171e' },
  'yml': { icon: 'settings-gear', color: '#cb171e' },
  'xml': { icon: 'code', color: '#f97316' },
  'toml': { icon: 'settings', color: '#9c4221' },
  'ini': { icon: 'settings', color: '#515151' },
  'env': { icon: 'gear', color: '#31859c' },
  'sql': { icon: 'database', color: '#f29111' },
  'db': { icon: 'database', color: '#f29111' },
  'sqlite': { icon: 'database', color: '#003b57' },
  
  // Documentation
  'md': { icon: 'markdown', color: '#519aba' },
  'mdx': { icon: 'markdown', color: '#519aba' },
  'rst': { icon: 'book', color: '#8c8c8c' },
  'txt': { icon: 'file-text', color: '#969696' },
  'pdf': { icon: 'file-pdf', color: '#dd0031' },
  'doc': { icon: 'file-binary', color: '#185abd' },
  'docx': { icon: 'file-binary', color: '#185abd' },
  
  // Shell & Scripts
  'sh': { icon: 'terminal', color: '#4eaa25' },
  'bash': { icon: 'terminal', color: '#4eaa25' },
  'zsh': { icon: 'terminal', color: '#4eaa25' },
  'fish': { icon: 'terminal', color: '#4eaa25' },
  'ps1': { icon: 'terminal-powershell', color: '#012456' },
  'bat': { icon: 'terminal-cmd', color: '#c1f12e' },
  'cmd': { icon: 'terminal-cmd', color: '#c1f12e' },
  
  // Build Tools
  'dockerfile': { icon: 'file-submodule', color: '#2496ed' },
  'makefile': { icon: 'tools', color: '#6d8086' },
  'cmake': { icon: 'tools', color: '#da3434' },
  'gradle': { icon: 'tools', color: '#02303a' },
  
  // Version Control
  'gitignore': { icon: 'git-commit', color: '#f05032' },
  'gitattributes': { icon: 'git-commit', color: '#f05032' },
  'gitmodules': { icon: 'git-commit', color: '#f05032' },
  
  // Images
  'png': { icon: 'file-media', color: '#a074c4' },
  'jpg': { icon: 'file-media', color: '#a074c4' },
  'jpeg': { icon: 'file-media', color: '#a074c4' },
  'gif': { icon: 'file-media', color: '#a074c4' },
  'svg': { icon: 'file-media', color: '#ffb13b' },
  'ico': { icon: 'file-media', color: '#b8e7c8' },
  'webp': { icon: 'file-media', color: '#a074c4' },
  
  // Archives
  'zip': { icon: 'file-zip', color: '#b8b816' },
  'tar': { icon: 'file-zip', color: '#b8b816' },
  'gz': { icon: 'file-zip', color: '#b8b816' },
  'rar': { icon: 'file-zip', color: '#b8b816' },
  '7z': { icon: 'file-zip', color: '#b8b816' },
  
  // Other
  'log': { icon: 'output', color: '#5a5a5a' },
  'lock': { icon: 'lock', color: '#8c8c8c' },
  'LICENSE': { icon: 'law', color: '#cc0000' },
  'test': { icon: 'beaker', color: '#00d4b8' },
  'spec': { icon: 'beaker', color: '#00d4b8' }
};

// Special file name mappings (exact matches)
export const fileNameMap: Record<string, IconMapping> = {
  'package.json': { icon: 'json', color: '#8bc34a' },
  'package-lock.json': { icon: 'lock', color: '#8bc34a' },
  'yarn.lock': { icon: 'lock', color: '#2c8ebb' },
  'pnpm-lock.yaml': { icon: 'lock', color: '#f69220' },
  'tsconfig.json': { icon: 'settings-gear', color: '#007acc' },
  'webpack.config.js': { icon: 'package', color: '#8dd6f9' },
  'vite.config.js': { icon: 'rocket', color: '#646cff' },
  'rollup.config.js': { icon: 'package', color: '#ff3333' },
  '.eslintrc': { icon: 'extensions', color: '#4b32c3' },
  '.eslintrc.js': { icon: 'extensions', color: '#4b32c3' },
  '.eslintrc.json': { icon: 'extensions', color: '#4b32c3' },
  '.prettierrc': { icon: 'symbol-color', color: '#f7b93e' },
  '.babelrc': { icon: 'symbol-misc', color: '#f9dc3e' },
  'Dockerfile': { icon: 'file-submodule', color: '#2496ed' },
  'docker-compose.yml': { icon: 'file-submodule', color: '#2496ed' },
  'Makefile': { icon: 'tools', color: '#6d8086' },
  'README.md': { icon: 'info', color: '#42a5f5' },
  'CHANGELOG.md': { icon: 'history', color: '#42a5f5' },
  'LICENSE': { icon: 'law', color: '#cc0000' },
  '.gitignore': { icon: 'git-commit', color: '#f05032' },
  '.env': { icon: 'key', color: '#31859c' },
  '.env.local': { icon: 'key', color: '#31859c' },
  '.env.development': { icon: 'key', color: '#31859c' },
  '.env.production': { icon: 'key', color: '#31859c' }
};

// Folder icon mappings
export const folderIconMap: Record<string, IconMapping> = {
  '.git': { icon: 'github', color: '#f05032' },
  '.vscode': { icon: 'settings-gear', color: '#007acc' },
  'node_modules': { icon: 'package', color: '#8bc34a' },
  'src': { icon: 'code', color: '#4285f4' },
  'dist': { icon: 'export', color: '#f77669' },
  'build': { icon: 'tools', color: '#f77669' },
  'out': { icon: 'export', color: '#f77669' },
  'public': { icon: 'globe', color: '#42a5f5' },
  'static': { icon: 'archive', color: '#42a5f5' },
  'assets': { icon: 'folder-opened', color: '#66bb6a' },
  'images': { icon: 'file-media', color: '#a074c4' },
  'styles': { icon: 'symbol-color', color: '#ec407a' },
  'css': { icon: 'symbol-color', color: '#1572b6' },
  'js': { icon: 'symbol-namespace', color: '#f1dd35' },
  'scripts': { icon: 'terminal', color: '#4eaa25' },
  'tests': { icon: 'beaker', color: '#00d4b8' },
  'test': { icon: 'beaker', color: '#00d4b8' },
  '__tests__': { icon: 'beaker', color: '#00d4b8' },
  'spec': { icon: 'beaker', color: '#00d4b8' },
  'docs': { icon: 'book', color: '#42a5f5' },
  'documentation': { icon: 'book', color: '#42a5f5' },
  'config': { icon: 'settings', color: '#9e9e9e' },
  'components': { icon: 'symbol-namespace', color: '#43a047' },
  'layouts': { icon: 'layout', color: '#43a047' },
  'pages': { icon: 'browser', color: '#43a047' },
  'api': { icon: 'plug', color: '#009688' },
  'lib': { icon: 'library', color: '#f06292' },
  'utils': { icon: 'tools', color: '#ff7043' },
  'helpers': { icon: 'lightbulb', color: '#ff7043' },
  'models': { icon: 'database', color: '#ff5722' },
  'controllers': { icon: 'server-process', color: '#9c27b0' },
  'views': { icon: 'eye', color: '#2196f3' },
  'routes': { icon: 'milestone', color: '#4caf50' },
  'middleware': { icon: 'layers', color: '#ff9800' },
  'migrations': { icon: 'database', color: '#795548' },
  'seeds': { icon: 'database', color: '#8bc34a' },
  'vendor': { icon: 'package', color: '#9e9e9e' },
  '.github': { icon: 'github', color: '#24292e' },
  'workflows': { icon: 'run-all', color: '#24292e' }
};

/**
 * Get icon for a file based on its name and extension
 */
export function getFileIcon(fileName: string): IconMapping {
  // Check exact filename matches first
  if (fileNameMap[fileName]) {
    return fileNameMap[fileName];
  }
  
  // Check file extension
  const ext = fileName.split('.').pop()?.toLowerCase();
  if (ext && fileIconMap[ext]) {
    return fileIconMap[ext];
  }
  
  // Default file icon
  return { icon: 'file', color: '#969696' };
}

/**
 * Get icon for a folder based on its name
 */
export function getFolderIcon(folderName: string, isExpanded: boolean = false): IconMapping {
  // Check special folder names
  if (folderIconMap[folderName]) {
    return folderIconMap[folderName];
  }
  
  // Default folder icon
  return { 
    icon: isExpanded ? 'folder-opened' : 'folder',
    color: '#dcb67a'
  };
}

/**
 * Create HTML for a file/folder icon
 */
export function createIconElement(iconMapping: IconMapping): string {
  return `<i class="codicon codicon-${iconMapping.icon}" style="color: ${iconMapping.color || '#969696'}"></i>`;
}