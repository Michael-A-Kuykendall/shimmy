#!/usr/bin/env node

import { fileURLToPath } from 'url';
import { dirname, join, resolve } from 'path';
import { readFileSync, writeFileSync, mkdirSync, existsSync, cpSync } from 'fs';
import { program } from 'commander';
import chalk from 'chalk';
import ejs from 'ejs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Schema loading function with path option and fallback
async function loadSchema(contractPath, schemaUrl) {
  // If a schema URL is provided (via CLI option or env), try that first
  if (schemaUrl) {
    try {
      console.log(chalk.yellow(`📡 Fetching schema from provided URL: ${schemaUrl}`));
      const response = await fetch(schemaUrl);
      if (response.ok) {
        const schema = await response.json();
        console.log(chalk.green('✅ Schema fetched from provided URL'));
        return schema;
      }
      console.log(chalk.yellow(`⚠️  Schema fetch returned HTTP ${response.status}`));
    } catch (error) {
      console.log(chalk.yellow('⚠️ Failed to fetch schema from provided URL:', error.message));
    }
  }
  let schemaPath;
  
  if (contractPath && existsSync(contractPath)) {
    schemaPath = resolve(contractPath);
  } else if (existsSync(resolve(__dirname, '../exported-shimmy-schema.json'))) {
    schemaPath = resolve(__dirname, '../exported-shimmy-schema.json');
  } else {
    // Try to fetch from running shimmy instance
    try {
      console.log(chalk.yellow('📡 Attempting to fetch schema from running shimmy instance...'));
      const response = await fetch('http://127.0.0.1:11435/__shimmy__/schema');
      if (response.ok) {
        const schema = await response.json();
        console.log(chalk.green('✅ Schema fetched from running shimmy'));
        return schema;
      }
    } catch (error) {
      console.log(chalk.yellow('⚠️ Failed to fetch schema from running shimmy'));
    }
    
    console.error(chalk.red('❌ No schema found. Tried:'));
    console.error(chalk.red('   - Provided path:', contractPath || 'none'));
    console.error(chalk.red('   - Default location: ../exported-shimmy-schema.json'));
    console.error(chalk.red('   - Running shimmy: http://127.0.0.1:11435/__shimmy__/schema'));
    process.exit(1);
  }

  try {
    const schemaContent = readFileSync(schemaPath, 'utf8');
    const schema = JSON.parse(schemaContent);
    console.log(chalk.green(`✅ Schema loaded from: ${schemaPath}`));
    return schema;
  } catch (error) {
    console.error(chalk.red('❌ Failed to load schema:'), error.message);
    process.exit(1);
  }
}

// Code Generation Logic for Phase 4.3
function generateTypeScriptInterfaces(schema) {
  const interfaces = [];
  
  if (schema.websocket_messages) {
    for (const message of schema.websocket_messages) {
      if (message.schema) {
        const interfaceName = message.name.split('_').map(word => 
          word.charAt(0).toUpperCase() + word.slice(1)
        ).join('');
        
        interfaces.push({
          name: interfaceName,
          description: message.description,
          schema: message.schema,
          originalName: message.name
        });
      }
    }
  }
  
  return interfaces;
}

function generateWebSocketHooks(schema) {
  const hooks = [];
  
  if (schema.websocket_messages) {
    const requestMessages = schema.websocket_messages.filter(m => m.type === 'request');
    const responseMessages = schema.websocket_messages.filter(m => m.type === 'response');
    
    for (const request of requestMessages) {
      const hookName = `use${request.name.split('_').map(word => 
        word.charAt(0).toUpperCase() + word.slice(1)
      ).join('')}`;
      
      // Find related response messages
      const relatedResponses = responseMessages.filter(r => {
        const requestBase = request.name.replace(/(_request|_req)$/, '');
        return r.name.includes(requestBase) || 
               r.name.includes(request.name.replace('get_', '').replace('_', '_response'));
      });
      
      hooks.push({
        name: hookName,
        requestMessage: request,
        responseMessages: relatedResponses,
        requestType: request.name
      });
    }
  }
  
  return hooks;
}

function generateUIComponents(schema) {
  const components = [];
  
  if (schema.required_behaviors) {
    for (const behavior of schema.required_behaviors) {
      const componentName = behavior.split('_').map(word => 
        word.charAt(0).toUpperCase() + word.slice(1)
      ).join('');
      
      components.push({
        name: componentName,
        behavior: behavior,
        description: `Component for ${behavior.replace(/_/g, ' ')}`,
        fileName: `${componentName}.tsx`
      });
    }
  }
  
  return components;
}

function generateDiscoveryClient(schema) {
  const discovery = schema.discovery || {};
  
  return {
    endpoint: discovery.endpoint || '/api/discovery',
    port: discovery.port || 11430,
    validationFields: discovery.validation_fields || ['backends'],
    timeout: discovery.timeout || 5000
  };
}

// Template processing function
async function processTemplate(templatePath, outputPath, data) {
  try {
    if (!existsSync(templatePath)) {
      console.error(chalk.red('❌ Template not found:'), templatePath);
      return false;
    }

    const templateContent = readFileSync(templatePath, 'utf8');
    const rendered = ejs.render(templateContent, data, { 
      filename: templatePath,
      async: false 
    });
    
    // Ensure output directory exists
    const outputDir = dirname(outputPath);
    mkdirSync(outputDir, { recursive: true });
    
    writeFileSync(outputPath, rendered, 'utf8');
    console.log(chalk.blue('📝 Generated:'), outputPath);
    return true;
  } catch (error) {
    console.error(chalk.red('❌ Failed to process template:'), templatePath);
    console.error(chalk.red('   Error:'), error.message);
    return false;
  }
}

// Copy static files
function copyStatic(sourcePath, targetPath) {
  try {
    if (existsSync(sourcePath)) {
      mkdirSync(dirname(targetPath), { recursive: true });
      cpSync(sourcePath, targetPath, { recursive: true });
      console.log(chalk.green('📋 Copied:'), targetPath);
      return true;
    }
    return false;
  } catch (error) {
    console.error(chalk.red('❌ Failed to copy:'), sourcePath);
    console.error(chalk.red('   Error:'), error.message);
    return false;
  }
}

// Generate React Vite theme with full code generation
async function generateReactViteTheme(themeName, outputDir, schema, options = {}) {
  console.log(chalk.cyan(`🎨 Generating React Vite theme: ${themeName}`));
  
  const templateDir = join(__dirname, 'templates', 'react-vite');
  const themeDir = join(outputDir, themeName);
  
  // Generate code elements from schema
  const typeScriptInterfaces = generateTypeScriptInterfaces(schema);
  const webSocketHooks = generateWebSocketHooks(schema);
  const uiComponents = generateUIComponents(schema);
  const discoveryClient = generateDiscoveryClient(schema);
  
  console.log(chalk.blue(`📋 Code generation summary:`));
  console.log(chalk.gray(`   TypeScript interfaces: ${typeScriptInterfaces.length}`));
  console.log(chalk.gray(`   WebSocket hooks: ${webSocketHooks.length}`));
  console.log(chalk.gray(`   UI components: ${uiComponents.length}`));
  console.log(chalk.gray(`   Discovery config: ${discoveryClient.endpoint}:${discoveryClient.port}`));
  
  // Enhanced template data with generated code
  const templateData = {
    // Basic theme info
    themeName,
    schema,
    packageName: themeName.toLowerCase().replace(/[^a-z0-9-]/g, '-'),
    description: `Shimmy frontend theme: ${themeName}`,
    author: options.author || 'Shimmy Team',
    version: '1.0.0',
    timestamp: new Date().toISOString(),
    
    // Generated code elements
    interfaces: typeScriptInterfaces,
    hooks: webSocketHooks,
    components: uiComponents,
    discovery: discoveryClient,
    
    // Schema data for templates
    websocketEndpoint: schema.streaming?.endpoint || '/ws/console',
    messageTypes: schema.websocket_messages || [],
    requiredBehaviors: schema.required_behaviors || [],
    
    // Helper functions for templates
    helpers: {
      capitalizeFirst: (str) => str.charAt(0).toUpperCase() + str.slice(1),
      camelCase: (str) => str.replace(/-([a-z])/g, (g) => g[1].toUpperCase()),
      pascalCase: (str) => str.split(/[-_]/).map(word => 
        word.charAt(0).toUpperCase() + word.slice(1)
      ).join(''),
      kebabCase: (str) => str.replace(/([a-z])([A-Z])/g, '$1-$2').toLowerCase(),
      jsonStringify: (obj) => JSON.stringify(obj, null, 2).replace(/</g, '&lt;').replace(/>/g, '&gt;')
    },
    
    ...options
  };

  let successCount = 0;
  let totalCount = 0;

  // Template files to process
  const templates = [
    { src: 'package.json.ejs', dest: 'package.json' },
    { src: 'vite.config.ts.ejs', dest: 'vite.config.ts' },
    { src: 'tsconfig.json.ejs', dest: 'tsconfig.json' },
    { src: 'tsconfig.node.json.ejs', dest: 'tsconfig.node.json' },
    { src: 'index.html.ejs', dest: 'index.html' },
    { src: 'src/main.tsx.ejs', dest: 'src/main.tsx' },
    { src: 'src/App.tsx.ejs', dest: 'src/App.tsx' },
    { src: 'src/hooks/useDiscovery.ts.ejs', dest: 'src/hooks/useDiscovery.ts' },
    { src: 'src/hooks/useWebSocket.ts.ejs', dest: 'src/hooks/useWebSocket.ts' },
    { src: 'src/components/ModelChooser.tsx.ejs', dest: 'src/components/ModelChooser.tsx' },
    { src: 'src/components/Chat.tsx.ejs', dest: 'src/components/Chat.tsx' },
    { src: 'src/components/Metrics.tsx.ejs', dest: 'src/components/Metrics.tsx' },
    { src: 'src/index.css.ejs', dest: 'src/index.css' },
    { src: 'tailwind.config.js.ejs', dest: 'tailwind.config.js' },
    { src: 'postcss.config.js.ejs', dest: 'postcss.config.js' },
    { src: 'README.md.ejs', dest: 'README.md' }
  ];

  // Process each template
  for (const template of templates) {
    totalCount++;
    const templatePath = join(templateDir, template.src);
    const outputPath = join(themeDir, template.dest);
    
    if (await processTemplate(templatePath, outputPath, templateData)) {
      successCount++;
    }
  }

  // Copy static files
  const staticFiles = [
    { src: 'public', dest: 'public' }
  ];

  for (const staticFile of staticFiles) {
    const sourcePath = join(templateDir, staticFile.src);
    const targetPath = join(themeDir, staticFile.dest);
    
    if (copyStatic(sourcePath, targetPath)) {
      // Static files don't count in template processing
    }
  }

  console.log(chalk.cyan('📊 Generation Summary:'));
  console.log(chalk.green(`   ✅ ${successCount}/${totalCount} templates processed successfully`));
  
  if (successCount === totalCount) {
    console.log(chalk.green(`🎉 Theme "${themeName}" generated successfully!`));
    console.log(chalk.blue(`📁 Location: ${themeDir}`));
    console.log(chalk.yellow('🚀 Next steps:'));
    console.log(chalk.yellow(`   cd ${themeDir}`));
    console.log(chalk.yellow('   npm install'));
    console.log(chalk.yellow('   npm run dev'));
    return true;
  } else {
    console.log(chalk.red(`❌ Theme generation completed with errors`));
    return false;
  }
}

// Add npm install function
async function installDependencies(themeDir) {
  console.log(chalk.blue('📦 Installing dependencies...'));
  
  try {
    const { spawn } = await import('child_process');
    
    return new Promise((resolve, reject) => {
      const npm = spawn('npm', ['install'], { 
        cwd: themeDir, 
        stdio: 'inherit',
        shell: true 
      });
      
      npm.on('close', (code) => {
        if (code === 0) {
          console.log(chalk.green('✅ Dependencies installed successfully!'));
          resolve();
        } else {
          reject(new Error(`npm install failed with code ${code}`));
        }
      });
      
      npm.on('error', (error) => {
        reject(error);
      });
    });
  } catch (error) {
    throw new Error(`Failed to spawn npm: ${error.message}`);
  }
}

// CLI Setup for Phase 4.4
program
  .name('shimmy-theme-gen')
  .description('Schema-driven theme generator for Shimmy frontends')
  .version('1.0.0');

program
  .command('new <theme-name>')
  .description('Generate a new Shimmy theme')
  .option('-t, --template <template>', 'Template to use (react-vite|react-nextjs|vanilla)', 'react-vite')
  .option('-o, --output <dir>', 'Output directory', './themes')
  .option('-c, --contract-path <path>', 'Path to contract schema JSON file (default: auto-fetch from running shimmy)')
  .option('--schema-url <url>', 'URL to contract schema (e.g. http://127.0.0.1:11430/__shimmy__/schema)')
  .option('-a, --author <author>', 'Theme author name', 'Shimmy Team')
  .option('--no-install', 'Skip npm install')
  .action(async (themeName, options) => {
    try {
      console.log(chalk.cyan.bold('🚀 Shimmy Theme Generator'));
      console.log(chalk.cyan('========================\n'));
      
      console.log(chalk.blue(`🎨 Creating theme: ${chalk.bold(themeName)}`));
      console.log(chalk.gray(`   Template: ${options.template}`));
      console.log(chalk.gray(`   Output: ${resolve(options.output)}`));
      console.log(chalk.gray(`   Contract: ${options.contractPath || 'auto-fetch'}\n`));
      
          // Prefer CLI --schema-url or environment SHIMMY_SCHEMA_URL over contract_path
          const schemaUrl = options.schemaUrl || process.env.SHIMMY_SCHEMA_URL;
          const schema = await loadSchema(options.contractPath, schemaUrl);
      const outputDir = resolve(options.output);
      
      let success = false;
      
      // Generate theme based on template
      switch (options.template) {
        case 'react-vite':
          success = await generateReactViteTheme(themeName, outputDir, schema, {
            author: options.author,
            install: false // We'll handle install separately
          });
          break;
          
        case 'react-nextjs':
          console.log(chalk.yellow('⚠️  react-nextjs template not yet implemented, using react-vite'));
          success = await generateReactViteTheme(themeName, outputDir, schema, {
            author: options.author,
            install: false
          });
          break;
          
        case 'vanilla':
          console.log(chalk.yellow('⚠️  vanilla template not yet implemented, using react-vite'));
          success = await generateReactViteTheme(themeName, outputDir, schema, {
            author: options.author,
            install: false
          });
          break;
          
        default:
          console.error(chalk.red(`❌ Unknown template: ${options.template}`));
          console.error(chalk.red('   Available templates: react-vite, react-nextjs, vanilla'));
          process.exit(1);
      }
      
      if (!success) {
        console.error(chalk.red('❌ Theme generation failed'));
        process.exit(1);
      }
      
      // Install dependencies if requested
      if (options.install) {
        const themeDir = join(outputDir, themeName);
        try {
          await installDependencies(themeDir);
        } catch (error) {
          console.log(chalk.yellow(`⚠️  Failed to install dependencies: ${error.message}`));
          console.log(chalk.yellow(`   You can install them manually by running:`));
          console.log(chalk.gray(`   cd ${themeDir} && npm install\n`));
        }
      }
      
      // Success message with next steps
      const themeDir = join(outputDir, themeName);
      console.log(chalk.green.bold(`\n🎉 Theme "${themeName}" created successfully!`));
      console.log(chalk.blue('\n📋 Next steps:'));
      console.log(chalk.gray(`   cd ${themeDir}`));
      if (!options.install) {
        console.log(chalk.gray('   npm install'));
      }
      console.log(chalk.gray('   npm run dev'));
      console.log(chalk.gray('\n🌐 Then open http://localhost:5173 in your browser'));
      
    } catch (error) {
      console.error(chalk.red.bold('\n❌ Generation failed:'));
      console.error(chalk.red(`   ${error.message}`));
      process.exit(1);
    }
  });

program
  .command('list-templates')
  .description('List available templates')
  .action(() => {
    console.log(chalk.cyan('Available Templates:'));
    console.log(chalk.blue('  react-vite') + '  - React + Vite + TypeScript theme');
  });

program
  .command('validate-schema [path]')
  .description('Validate a Shimmy frontend contract schema')
  .action(async (schemaPath) => {
    try {
      console.log(chalk.cyan('🔍 Validating Schema...'));
      const schema = await loadSchema(schemaPath);
      
      // Basic validation checks
      const required = ['discovery', 'required_behaviors', 'streaming', 'version', 'websocket_messages'];
      const missing = required.filter(field => !schema[field]);
      
      if (missing.length > 0) {
        console.log(chalk.red('❌ Missing required fields:'), missing.join(', '));
        process.exit(1);
      }
      
      console.log(chalk.green('✅ Schema validation passed'));
      console.log(chalk.blue(`   Version: ${schema.version}`));
      console.log(chalk.blue(`   WebSocket messages: ${schema.websocket_messages?.length || 0}`));
      console.log(chalk.blue(`   Required behaviors: ${schema.required_behaviors?.length || 0}`));
      console.log(chalk.blue(`   Discovery endpoint: ${schema.discovery?.endpoint || 'not specified'}`));
    } catch (error) {
      console.error(chalk.red('❌ Schema validation failed:'), error.message);
      process.exit(1);
    }
  });

// Parse CLI arguments
if (process.argv.length <= 2) {
  program.help();
}

program.parse();