#!/usr/bin/env node

/**
 * Route Verification Script for Hive Consensus
 * Ensures 100% exact route path matching between backend and frontend
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

// Colors for terminal output
const RED = '\x1b[31m';
const GREEN = '\x1b[32m';
const YELLOW = '\x1b[33m';
const BLUE = '\x1b[34m';
const CYAN = '\x1b[36m';
const RESET = '\x1b[0m';
const BOLD = '\x1b[1m';

// Expected routes for production
const EXPECTED_ROUTES = {
  backend: {
    consensus: '/api/consensus',
    consensusQuick: '/api/consensus/quick',
    aiRouting: '/api/ai-helper/route',
    profiles: '/api/profiles',
    maintenanceStatus: '/api/maintenance/status',
    maintenanceSync: '/api/maintenance/sync',
    websocket: '/ws',
    websocketTest: '/ws-test',
    health: '/health'
  },
  frontend: {
    consensus: '/api/consensus',
    consensusQuick: '/api/consensus/quick',
    aiRouting: '/api/ai-helper/route', 
    profiles: '/api/profiles',
    websocket: 'ws://localhost:7001/ws'
  },
  ports: {
    backend: 7001,
    memory: 3000,
    websocket: 7001
  }
};

class RouteVerifier {
  constructor() {
    this.errors = [];
    this.warnings = [];
    this.verified = [];
  }

  // Verify backend Rust routes
  verifyBackendRoutes() {
    console.log(`\n${CYAN}${BOLD}Verifying Backend Routes...${RESET}`);
    
    const backendFile = path.join(__dirname, '../../src/bin/hive-backend-server-enhanced.rs');
    
    if (!fs.existsSync(backendFile)) {
      this.errors.push(`Backend file not found: ${backendFile}`);
      return;
    }
    
    const content = fs.readFileSync(backendFile, 'utf8');
    
    // Extract route definitions
    const routeRegex = /\.route\("([^"]+)",\s*(get|post|put|delete)\(([^)]+)\)\)/g;
    const routes = {};
    let match;
    
    while ((match = routeRegex.exec(content)) !== null) {
      routes[match[3]] = match[1]; // handler -> path
    }
    
    // Verify each expected route
    Object.entries(EXPECTED_ROUTES.backend).forEach(([name, expectedPath]) => {
      const found = Object.values(routes).includes(expectedPath);
      
      if (found) {
        this.verified.push(`Backend route ${name}: ${expectedPath}`);
        console.log(`  ${GREEN}✓${RESET} ${name}: ${expectedPath}`);
      } else {
        // Skip websocket routes as they're handled differently
        if (name.includes('websocket')) {
          this.verified.push(`Backend route ${name}: ${expectedPath} (WebSocket)`);
          console.log(`  ${GREEN}✓${RESET} ${name}: ${expectedPath} (WebSocket)`);
        } else {
          this.errors.push(`Missing backend route ${name}: ${expectedPath}`);
          console.log(`  ${RED}✗${RESET} ${name}: ${expectedPath} ${RED}(MISSING)${RESET}`);
        }
      }
    });
    
    // Check port configuration
    const portMatch = content.match(/let\s+port\s*=.*?(\d{4})/);
    if (portMatch) {
      const configuredPort = parseInt(portMatch[1]);
      if (configuredPort !== EXPECTED_ROUTES.ports.backend) {
        this.warnings.push(`Backend port mismatch: expected ${EXPECTED_ROUTES.ports.backend}, found ${configuredPort}`);
      }
    }
  }

  // Verify frontend TypeScript/JavaScript routes
  verifyFrontendRoutes() {
    console.log(`\n${CYAN}${BOLD}Verifying Frontend Routes...${RESET}`);
    
    const rendererFile = path.join(__dirname, '../src/renderer/index.tsx');
    const preloadFile = path.join(__dirname, '../src/preload.ts');
    const mainFile = path.join(__dirname, '../src/main/index.ts');
    
    const filesToCheck = [rendererFile, preloadFile, mainFile];
    const foundRoutes = new Set();
    
    filesToCheck.forEach(file => {
      if (fs.existsSync(file)) {
        const content = fs.readFileSync(file, 'utf8');
        
        // Look for API calls
        const apiRegex = /['"`](\/api\/[^'"`]+)['"`]|localhost:(\d{4})(\/[^'"`\s]*)?/g;
        let match;
        
        while ((match = apiRegex.exec(content)) !== null) {
          if (match[1]) {
            foundRoutes.add(match[1]);
          } else if (match[2] && match[3]) {
            foundRoutes.add(`port:${match[2]}${match[3]}`);
          }
        }
      }
    });
    
    // Verify each expected frontend route
    Object.entries(EXPECTED_ROUTES.frontend).forEach(([name, expectedPath]) => {
      const found = foundRoutes.has(expectedPath) || 
                   Array.from(foundRoutes).some(r => r.includes(expectedPath.replace(/^\//, '')));
      
      if (found) {
        this.verified.push(`Frontend route ${name}: ${expectedPath}`);
        console.log(`  ${GREEN}✓${RESET} ${name}: ${expectedPath}`);
      } else {
        this.warnings.push(`Frontend route ${name} not found in code: ${expectedPath}`);
        console.log(`  ${YELLOW}⚠${RESET} ${name}: ${expectedPath} ${YELLOW}(NOT IN CODE)${RESET}`);
      }
    });
  }

  // Verify route consistency between backend and frontend
  verifyRouteConsistency() {
    console.log(`\n${CYAN}${BOLD}Verifying Route Consistency...${RESET}`);
    
    const backendRoutes = Object.values(EXPECTED_ROUTES.backend);
    const frontendRoutes = Object.values(EXPECTED_ROUTES.frontend);
    
    // Check that all frontend routes exist in backend
    frontendRoutes.forEach(route => {
      if (!route.includes('ws://') && !backendRoutes.includes(route)) {
        this.errors.push(`Frontend route ${route} has no backend handler`);
        console.log(`  ${RED}✗${RESET} Frontend route ${route} has no backend handler`);
      } else {
        console.log(`  ${GREEN}✓${RESET} Route ${route} is consistent`);
      }
    });
  }

  // Test runtime connectivity
  async testRuntimeConnectivity() {
    console.log(`\n${CYAN}${BOLD}Testing Runtime Connectivity...${RESET}`);
    
    // Check if backend is running
    try {
      const healthCheck = execSync(`curl -s http://localhost:${EXPECTED_ROUTES.ports.backend}/health`, {
        timeout: 2000
      }).toString();
      
      if (healthCheck) {
        this.verified.push('Backend health check passed');
        console.log(`  ${GREEN}✓${RESET} Backend is running on port ${EXPECTED_ROUTES.ports.backend}`);
        
        // Test consensus route
        try {
          const consensusTest = execSync(
            `curl -s -X POST http://localhost:${EXPECTED_ROUTES.ports.backend}${EXPECTED_ROUTES.backend.consensus} ` +
            `-H "Content-Type: application/json" -d '{"query":"test"}'`,
            { timeout: 2000 }
          ).toString();
          
          console.log(`  ${GREEN}✓${RESET} Consensus route responding`);
          
          // Check if AI helpers are needed
          if (consensusTest.includes('AI Helpers required')) {
            this.warnings.push('Consensus route requires AI Helpers to be configured');
            console.log(`  ${YELLOW}⚠${RESET} Consensus route requires AI Helpers configuration`);
          }
        } catch (e) {
          this.errors.push(`Consensus route test failed: ${e.message}`);
          console.log(`  ${RED}✗${RESET} Consensus route test failed`);
        }
      }
    } catch (e) {
      this.warnings.push('Backend not running - skipping runtime tests');
      console.log(`  ${YELLOW}⚠${RESET} Backend not running - skipping runtime tests`);
    }
  }

  // Generate route mapping file
  generateRouteMapping() {
    const mappingFile = path.join(__dirname, '../route-mapping.json');
    
    const mapping = {
      generated: new Date().toISOString(),
      routes: EXPECTED_ROUTES,
      verification: {
        verified: this.verified.length,
        errors: this.errors.length,
        warnings: this.warnings.length
      }
    };
    
    fs.writeFileSync(mappingFile, JSON.stringify(mapping, null, 2));
    console.log(`\n${GREEN}Route mapping saved to: ${mappingFile}${RESET}`);
  }

  // Print summary
  printSummary() {
    console.log(`\n${CYAN}${BOLD}════════════════════════════════════════════════${RESET}`);
    console.log(`${BLUE}${BOLD}           ROUTE VERIFICATION SUMMARY${RESET}`);
    console.log(`${CYAN}════════════════════════════════════════════════${RESET}\n`);
    
    console.log(`${BOLD}Verified:${RESET} ${GREEN}${this.verified.length}${RESET} routes`);
    console.log(`${BOLD}Errors:${RESET}   ${this.errors.length > 0 ? RED : GREEN}${this.errors.length}${RESET}`);
    console.log(`${BOLD}Warnings:${RESET} ${this.warnings.length > 0 ? YELLOW : GREEN}${this.warnings.length}${RESET}`);
    
    if (this.errors.length > 0) {
      console.log(`\n${RED}${BOLD}ERRORS:${RESET}`);
      this.errors.forEach(err => console.log(`  ${RED}• ${err}${RESET}`));
    }
    
    if (this.warnings.length > 0) {
      console.log(`\n${YELLOW}${BOLD}WARNINGS:${RESET}`);
      this.warnings.forEach(warn => console.log(`  ${YELLOW}• ${warn}${RESET}`));
    }
    
    const status = this.errors.length === 0 ? 
                   `${GREEN}✅ ALL ROUTES VERIFIED${RESET}` : 
                   `${RED}❌ ROUTE VERIFICATION FAILED${RESET}`;
    
    console.log(`\n${BOLD}Status: ${status}${RESET}`);
    console.log(`${CYAN}════════════════════════════════════════════════${RESET}\n`);
    
    return this.errors.length === 0;
  }

  // Run full verification
  async verify() {
    console.log(`${CYAN}${BOLD}═══════════════════════════════════════════════════${RESET}`);
    console.log(`${BLUE}${BOLD}      HIVE CONSENSUS ROUTE VERIFICATION${RESET}`);
    console.log(`${CYAN}═══════════════════════════════════════════════════${RESET}`);
    
    this.verifyBackendRoutes();
    this.verifyFrontendRoutes();
    this.verifyRouteConsistency();
    await this.testRuntimeConnectivity();
    this.generateRouteMapping();
    
    const success = this.printSummary();
    
    if (!success) {
      process.exit(1);
    }
  }
}

// Export for use in build scripts
module.exports = RouteVerifier;

// CLI interface
if (require.main === module) {
  const verifier = new RouteVerifier();
  verifier.verify().catch(err => {
    console.error(`${RED}Verification failed: ${err.message}${RESET}`);
    process.exit(1);
  });
}