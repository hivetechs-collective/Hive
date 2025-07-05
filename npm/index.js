#!/usr/bin/env node

// HiveTechs Consensus NPM Package
// This is a CLI tool - use the 'hive' command directly

module.exports = {
  name: '@hivetechs/hive',
  version: '2.1.0',
  description: 'HiveTechs Consensus - The world\'s most advanced AI-powered development assistant',
  cli: true,
  binary: 'hive'
};

// If run directly, show usage
if (require.main === module) {
  console.log('🐝 HiveTechs Consensus');
  console.log('');
  console.log('This is a CLI tool. Please use the \'hive\' command directly:');
  console.log('');
  console.log('  hive --help     Show all available commands');
  console.log('  hive ask        Ask AI anything');
  console.log('  hive analyze    Analyze your codebase');
  console.log('  hive tui        Launch interactive interface');
  console.log('');
  console.log('For more information: https://docs.hivetechs.com');
}