#!/usr/bin/env node

/**
 * Shimmy Theme Validator - Schema-driven validation for Shimmy themes
 * Phase 3.1-3.2 Implementation: Validator Infrastructure + Discovery Test
 */

const Ajv = require('ajv');
const addFormats = require('ajv-formats');
const fs = require('fs').promises;
const path = require('path');
const WebSocket = require('ws');

class ShimmyValidator {
    constructor() {
        this.ajv = new Ajv({ allErrors: true, strict: false });
        addFormats(this.ajv);
        this.schema = null;
        this.discoveryValidator = null;
        this.modelsListValidator = null;
        this.modelSelectedValidator = null;
        this.chatTokenValidator = null;
        this.generationCompleteValidator = null;
        this.ws = null;
        this.results = {
            discovery: { status: 'pending', port: null, error: null },
            schema_load: { status: 'pending', error: null },
            websocket_connection: { status: 'pending', error: null },
            get_models_test: { status: 'pending', error: null, response: null },
            select_model_test: { status: 'pending', error: null, response: null },
            chat_test: { status: 'pending', error: null, tokens_received: 0, generation_complete: false }
        };
    }

    /**
     * Load and compile the JSON schema
     */
    async loadSchema(schemaUrl) {
        try {
            // Determine schema source: explicit argument -> env -> discovery -> local file
            const envUrl = process.env.SHIMMY_SCHEMA_URL;
            const sourceUrl = schemaUrl || envUrl || null;

            if (sourceUrl) {
                console.log(`🌐 Loading schema from URL: ${sourceUrl}`);
                // Fetch schema over HTTP(S)
                const { default: fetch } = await import('node-fetch').catch(() => ({ default: global.fetch }));
                const res = await fetch(sourceUrl, { method: 'GET' });
                if (!res.ok) throw new Error(`Failed to fetch schema: HTTP ${res.status}`);
                this.schema = await res.json();
            } else {
                // If no explicit schema URL use discovery to find backend and fetch /__shimmy__/schema
                try {
                    const discoveryUrl = process.env.SHIMMY_DISCOVERY_URL || 'http://127.0.0.1:11430/api/discovery';
                    console.log(`📡 Attempting discovery to fetch schema via ${discoveryUrl}`);

                    const discoveryRes = await fetch(discoveryUrl, { method: 'GET' });
                    if (discoveryRes.ok) {
                        const discoveryData = await discoveryRes.json();
                        // Prefer a healthy backend
                        const backend = (discoveryData.backends || []).find(b => b.health && b.health.healthy) || (discoveryData.backends || [])[0];
                        if (backend && backend.port) {
                            const schemaEndpoint = `http://127.0.0.1:${backend.port}/__shimmy__/schema`;
                            console.log(`🔎 Fetching schema from backend: ${schemaEndpoint}`);
                            const schemaRes = await fetch(schemaEndpoint, { method: 'GET' });
                            if (schemaRes.ok) {
                                this.schema = await schemaRes.json();
                            }
                        }
                    }
                } catch (dErr) {
                    // Ignore; we'll fall back to local file below
                    console.log('⚠️ Discovery-based schema fetch failed:', dErr.message);
                }

                // If still not loaded, try local file fallback
                if (!this.schema) {
                    const schemaPath = path.resolve(__dirname, '..', 'exported-shimmy-schema.json');
                    const schemaContent = await fs.readFile(schemaPath, 'utf8');
                    this.schema = JSON.parse(schemaContent);
                }
            }
            
            // Create DiscoverySnapshot schema manually since it's not in the contract schema
            // Based on the actual Rust DiscoverySnapshot struct
            const discoverySchema = {
                type: "object",
                required: ["discovery_port", "last_updated", "epoch", "backends"],
                properties: {
                    discovery_port: { type: "integer", minimum: 1, maximum: 65535 },
                    last_updated: { type: "string", format: "date-time" },
                    epoch: { type: "integer", minimum: 0 },
                    backends: {
                        type: "array",
                        items: {
                            type: "object",
                            required: ["id", "url", "port", "models", "capabilities", "health", "started_at", "pid"],
                            properties: {
                                id: { type: "string" },
                                url: { type: "string", format: "uri" },
                                port: { type: "integer", minimum: 1, maximum: 65535 },
                                models: {
                                    type: "array",
                                    items: {
                                        type: "object",
                                        required: ["name", "display_name", "source", "backend", "health_status"],
                                        properties: {
                                            name: { type: "string" },
                                            display_name: { type: "string" },
                                            size_bytes: { type: ["integer", "null"], minimum: 0 },
                                            parameter_count: { type: ["string", "null"] },
                                            quantization: { type: ["string", "null"] },
                                            context_length: { type: ["integer", "null"], minimum: 1 },
                                            model_type: { type: ["string", "null"] },
                                            source: { type: "string" },
                                            backend: { type: "string" },
                                            health_status: { type: "string", enum: ["ready", "checking", "failed", "unknown"] },
                                            health_error: { type: ["string", "null"] }
                                        },
                                        additionalProperties: false
                                    }
                                },
                                capabilities: {
                                    type: "array",
                                    items: { type: "string" }
                                },
                                health: {
                                    type: "object",
                                    required: ["healthy", "last_check"],
                                    properties: {
                                        healthy: { type: "boolean" },
                                        last_check: { type: "string", format: "date-time" }
                                    },
                                    additionalProperties: false
                                },
                                started_at: { type: "string", format: "date-time" },
                                pid: { type: "integer", minimum: 1 }
                            },
                            additionalProperties: false
                        }
                    }
                },
                additionalProperties: false
            };
            
            // Compile the discovery validator
            this.discoveryValidator = this.ajv.compile(discoverySchema);
            
            // Create WebSocket message schemas for Phase 3.3-3.4 (simplified for validation)
            const wsSchemas = {
                ModelsListResponse: {
                    type: "object",
                    properties: {
                        type: { const: "models_response" },
                        success: { type: "boolean" },
                        models: { 
                            type: "array",
                            items: { type: "object" }  // Simplified - just check it's an array of objects
                        },
                        timestamp: { type: "string", format: "date-time" }
                    },
                    required: ["type", "success", "models", "timestamp"]
                },
                ModelSelectedResponse: {
                    type: "object", 
                    properties: {
                        type: { const: "model_selected" },
                        success: { type: "boolean" },
                        model_name: { type: "string" },
                        timestamp: { type: "string", format: "date-time" }
                    },
                    required: ["type", "success", "model_name", "timestamp"]
                },
                ChatTokenResponse: {
                    type: "object",
                    properties: {
                        type: { const: "token" },
                        token: { type: "string" },
                        is_final: { type: "boolean" }
                    },
                    required: ["token"]
                },
                GenerationCompleteResponse: {
                    type: "object",
                    properties: {
                        type: { const: "generation_complete" },
                        total_tokens: { type: "integer" },
                        generation_time_ms: { type: "integer" },
                        tokens_per_second: { type: "number" }
                    },
                    required: ["type", "total_tokens", "generation_time_ms", "tokens_per_second"]
                }
            };
            
            // Compile validators for WebSocket messages (Phase 3.3-3.4)
            this.modelsListValidator = this.ajv.compile(wsSchemas.ModelsListResponse);
            this.modelSelectedValidator = this.ajv.compile(wsSchemas.ModelSelectedResponse);
            this.chatTokenValidator = this.ajv.compile(wsSchemas.ChatTokenResponse);
            this.generationCompleteValidator = this.ajv.compile(wsSchemas.GenerationCompleteResponse);
            
            this.results.schema_load.status = 'success';
            console.log('✅ Discovery and WebSocket schemas loaded successfully');
        } catch (error) {
            this.results.schema_load.status = 'failed';
            this.results.schema_load.error = error.message;
            console.error('❌ Failed to load schema:', error.message);
            throw error;
        }
    }

    /**
     * Test HTTP discovery on 127.0.0.1:11430
     */
    async testDiscovery() {
        console.log('🔍 Testing HTTP discovery on 127.0.0.1:11430...');
        
        try {
            const startTime = Date.now();
            
            // Test discovery endpoint
            const discoveryUrl = process.env.SHIMMY_DISCOVERY_URL || 'http://127.0.0.1:11430/api/discovery';
            console.log(`🔎 Using discovery URL: ${discoveryUrl}`);

            // Retry loop: default is to retry indefinitely unless SHIMMY_VALIDATOR_MAX_RETRIES is set
            let response = null;
            const maxAttemptsEnv = process.env.SHIMMY_VALIDATOR_MAX_RETRIES;
            const maxAttempts = maxAttemptsEnv ? parseInt(maxAttemptsEnv, 10) : null; // null => infinite
            let attempt = 1;
            while (true) {
                try {
                    response = await fetch(discoveryUrl, {
                        method: 'GET',
                        headers: { 'Accept': 'application/json' },
                        signal: AbortSignal.timeout(10000)
                    });

                    if (!response.ok) {
                        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
                    }

                    // Successful fetch — stop retrying
                    break;
                } catch (err) {
                    // If maxAttempts is set and we've reached it, give up
                    if (maxAttempts && attempt >= maxAttempts) {
                        console.log(`❌ Discovery fetch attempt ${attempt} failed and reached max attempts (${maxAttempts}): ${err.message}`);
                        throw err;
                    }

                    // Otherwise, sleep and retry (graceful backoff)
                    console.log(`⏳ Discovery fetch attempt ${attempt} failed: ${err.message} — retrying`);
                    await new Promise(r => setTimeout(r, 1000));
                    attempt++;
                    continue;
                }
            }

            const discoveryData = await response.json();
            const duration = Date.now() - startTime;
            
            console.log(`📡 Discovery response received in ${duration}ms:`, JSON.stringify(discoveryData, null, 2));

            // Validate response against schema
            const isValid = this.discoveryValidator(discoveryData);
            
            if (!isValid) {
                const errors = this.discoveryValidator.errors;
                console.error('❌ Discovery response validation failed:', errors);
                this.results.discovery.status = 'validation_failed';
                this.results.discovery.error = `Schema validation errors: ${JSON.stringify(errors)}`;
                return;
            }

            console.log('✅ Discovery response validated against schema');

            // Extract backend port
            if (discoveryData.backends && discoveryData.backends.length > 0) {
                // Prefer healthy backend, fallback to first available
                const healthyBackend = discoveryData.backends.find(b => 
                    b.health && b.health.healthy === true
                );
                
                const selectedBackend = healthyBackend || discoveryData.backends[0];
                
                if (selectedBackend && selectedBackend.port) {
                    this.results.discovery.status = 'success';
                    this.results.discovery.port = selectedBackend.port;
                    console.log(`✅ Backend port extracted: ${selectedBackend.port}`);
                    
                    if (healthyBackend) {
                        console.log('✅ Selected healthy backend');
                    } else {
                        console.log('⚠️  Selected first available backend (health unknown)');
                    }
                } else {
                    throw new Error('No backend port found in discovery response');
                }
            } else {
                throw new Error('No backends found in discovery response');
            }

        } catch (error) {
            console.error('❌ Discovery test failed:', error.message);
            this.results.discovery.status = 'failed';
            this.results.discovery.error = error.message;
        }
    }

    /**
     * Test WebSocket connection (Phase 3.3)
     */
    async testWebSocketConnection() {
        if (!this.results.discovery.port) {
            this.results.websocket_connection.status = 'failed';
            this.results.websocket_connection.error = 'No backend port available from discovery';
            console.error('❌ WebSocket connection test skipped: No backend port');
            return;
        }

        console.log(`🔌 Testing WebSocket connection to ws://127.0.0.1:${this.results.discovery.port}/ws/console...`);
        
        return new Promise((resolve) => {
            const wsUrl = `ws://127.0.0.1:${this.results.discovery.port}/ws/console`;
            const ws = new WebSocket(wsUrl);
            
            // Set 5-second connection timeout
            const timeout = setTimeout(() => {
                this.results.websocket_connection.status = 'failed';
                this.results.websocket_connection.error = 'Connection timeout (5s)';
                console.error('❌ WebSocket connection timeout');
                ws.close();
                resolve();
            }, 5000);
            
            ws.on('open', () => {
                clearTimeout(timeout);
                this.results.websocket_connection.status = 'success';
                console.log('✅ WebSocket connected successfully');
                
                // Store the connection for message protocol tests
                this.ws = ws;
                resolve();
            });
            
            ws.on('error', (error) => {
                clearTimeout(timeout);
                this.results.websocket_connection.status = 'failed';
                this.results.websocket_connection.error = error.message;
                console.error('❌ WebSocket connection failed:', error.message);
                resolve();
            });
        });
    }

    /**
     * Test message protocols (Phase 3.4)
     */
    async testMessageProtocols() {
        if (!this.ws || this.results.websocket_connection.status !== 'success') {
            console.log('⚠️ Skipping message protocol tests: No WebSocket connection');
            return;
        }

        console.log('📡 Testing WebSocket message protocols...');
        
        // Test get_models message
        await this.testGetModels();
        
        // Test select_model message (only if get_models succeeded and we have models)
        if (this.results.get_models_test.status === 'success' && 
            this.results.get_models_test.response &&
            this.results.get_models_test.response.models &&
            this.results.get_models_test.response.models.length > 0) {
            
            const firstModel = this.results.get_models_test.response.models[0];
            await this.testSelectModel(firstModel.name);
            
            // Test chat message (only if select_model succeeded)
            if (this.results.select_model_test.status === 'success') {
                await this.testChat();
            }
        }
        
        // Close WebSocket when done
        this.ws.close();
    }

    /**
     * Test get_models message
     */
    async testGetModels() {
        console.log('🔍 Testing get_models message...');
        
        return new Promise((resolve) => {
            const messageHandler = (data) => {
                try {
                    const message = JSON.parse(data.toString());
                    
                    if (message.type === 'models_response') {
                        // Validate response against ModelsListResponse schema
                        const isValid = this.modelsListValidator(message);
                        
                        if (!isValid) {
                            const errors = this.modelsListValidator.errors;
                            this.results.get_models_test.status = 'failed';
                            this.results.get_models_test.error = `Schema validation failed: ${JSON.stringify(errors)}`;
                            console.error('❌ get_models response validation failed:', errors);
                        } else {
                            // Verify at least 1 model present
                            const modelCount = message.models ? message.models.length : 0;
                            if (modelCount === 0) {
                                this.results.get_models_test.status = 'failed';
                                this.results.get_models_test.error = 'No models found in response';
                                console.error('❌ get_models test failed: No models present');
                            } else {
                                this.results.get_models_test.status = 'success';
                                this.results.get_models_test.response = message;
                                console.log(`✅ get_models test PASS: Found ${modelCount} models, schema validation successful`);
                            }
                        }
                        
                        this.ws.removeListener('message', messageHandler);
                        resolve();
                    }
                } catch (error) {
                    this.results.get_models_test.status = 'failed';
                    this.results.get_models_test.error = `Message parsing error: ${error.message}`;
                    console.error('❌ get_models test failed: Message parsing error:', error.message);
                    this.ws.removeListener('message', messageHandler);
                    resolve();
                }
            };
            
            this.ws.on('message', messageHandler);
            
            // Send get_models request
            const getModelsMessage = {
                type: 'get_models',
                timestamp: new Date().toISOString()
            };
            
            this.ws.send(JSON.stringify(getModelsMessage));
            
            // Set timeout for response
            setTimeout(() => {
                if (this.results.get_models_test.status === 'pending') {
                    this.results.get_models_test.status = 'failed';
                    this.results.get_models_test.error = 'Response timeout';
                    console.error('❌ get_models test failed: Response timeout');
                    this.ws.removeListener('message', messageHandler);
                    resolve();
                }
            }, 10000); // 10 second timeout
        });
    }

    /**
     * Test select_model message
     */
    async testSelectModel(modelName) {
        console.log(`🎯 Testing select_model message with model: ${modelName}...`);
        
        return new Promise((resolve) => {
            const messageHandler = (data) => {
                try {
                    const message = JSON.parse(data.toString());
                    
                    if (message.type === 'model_selected') {
                        // Validate response against ModelSelectedResponse schema
                        const isValid = this.modelSelectedValidator(message);
                        
                        if (!isValid) {
                            const errors = this.modelSelectedValidator.errors;
                            this.results.select_model_test.status = 'failed';
                            this.results.select_model_test.error = `Schema validation failed: ${JSON.stringify(errors)}`;
                            console.error('❌ select_model response validation failed:', errors);
                        } else {
                            this.results.select_model_test.status = 'success';
                            this.results.select_model_test.response = message;
                            console.log('✅ select_model test PASS: Schema validation successful');
                        }
                        
                        this.ws.removeListener('message', messageHandler);
                        resolve();
                    }
                } catch (error) {
                    this.results.select_model_test.status = 'failed';
                    this.results.select_model_test.error = `Message parsing error: ${error.message}`;
                    console.error('❌ select_model test failed: Message parsing error:', error.message);
                    this.ws.removeListener('message', messageHandler);
                    resolve();
                }
            };
            
            this.ws.on('message', messageHandler);
            
            // Send select_model request
            const selectModelMessage = {
                type: 'select_model',
                model_name: modelName,
                timestamp: new Date().toISOString()
            };
            
            this.ws.send(JSON.stringify(selectModelMessage));
            
            // Set timeout for response
            setTimeout(() => {
                if (this.results.select_model_test.status === 'pending') {
                    this.results.select_model_test.status = 'failed';
                    this.results.select_model_test.error = 'Response timeout';
                    console.error('❌ select_model test failed: Response timeout');
                    this.ws.removeListener('message', messageHandler);
                    resolve();
                }
            }, 10000); // 10 second timeout
        });
    }

    /**
     * Test chat message with streaming tokens
     */
    async testChat() {
        console.log('💬 Testing chat message with streaming tokens...');
        
        return new Promise((resolve) => {
            let tokenCount = 0;
            let generationCompleteReceived = false;
            
            const messageHandler = (data) => {
                try {
                    const message = JSON.parse(data.toString());
                    
                    if (message.type === 'token') {
                        // Validate streaming token against ChatTokenResponse schema
                        const isValid = this.chatTokenValidator(message);
                        
                        if (!isValid) {
                            const errors = this.chatTokenValidator.errors;
                            this.results.chat_test.status = 'failed';
                            this.results.chat_test.error = `Token schema validation failed: ${JSON.stringify(errors)}`;
                            console.error('❌ Chat token validation failed:', errors);
                            this.ws.removeListener('message', messageHandler);
                            resolve();
                            return;
                        }
                        
                        tokenCount++;
                        this.results.chat_test.tokens_received = tokenCount;
                        
                    } else if (message.type === 'generation_complete') {
                        // Validate generation complete response
                        const isValid = this.generationCompleteValidator(message);
                        
                        if (!isValid) {
                            const errors = this.generationCompleteValidator.errors;
                            this.results.chat_test.status = 'failed';
                            this.results.chat_test.error = `Generation complete schema validation failed: ${JSON.stringify(errors)}`;
                            console.error('❌ Generation complete validation failed:', errors);
                        } else {
                            generationCompleteReceived = true;
                            this.results.chat_test.generation_complete = true;
                            this.results.chat_test.status = 'success';
                            console.log(`✅ chat test PASS: Received ${tokenCount} tokens, generation complete received and validated`);
                        }
                        
                        this.ws.removeListener('message', messageHandler);
                        resolve();
                    }
                } catch (error) {
                    this.results.chat_test.status = 'failed';
                    this.results.chat_test.error = `Message parsing error: ${error.message}`;
                    console.error('❌ chat test failed: Message parsing error:', error.message);
                    this.ws.removeListener('message', messageHandler);
                    resolve();
                }
            };
            
            this.ws.on('message', messageHandler);
            
            // Send chat request
            const chatMessage = {
                type: 'chat',
                message: 'Hello, this is a test message for validation.',
                timestamp: new Date().toISOString()
            };
            
            this.ws.send(JSON.stringify(chatMessage));
            
            // Set timeout for response
            setTimeout(() => {
                if (this.results.chat_test.status === 'pending') {
                    if (tokenCount > 0 && !generationCompleteReceived) {
                        this.results.chat_test.status = 'failed';
                        this.results.chat_test.error = `Received ${tokenCount} tokens but no generation_complete message`;
                        console.error(`❌ chat test failed: Received ${tokenCount} tokens but no generation_complete message`);
                    } else if (tokenCount === 0) {
                        this.results.chat_test.status = 'failed';
                        this.results.chat_test.error = 'No tokens received (timeout)';
                        console.error('❌ chat test failed: No tokens received (timeout)');
                    }
                    this.ws.removeListener('message', messageHandler);
                    resolve();
                }
            }, 30000); // 30 second timeout for chat
        });
    }

    /**
     * Generate test report
     */
    generateReport() {
        const report = {
            timestamp: new Date().toISOString(),
            phase: '3.1-3.4',
            description: 'Validator Infrastructure, Discovery Test, WebSocket Connection and Message Protocol Tests',
            results: this.results,
            summary: {
                schema_loaded: this.results.schema_load.status === 'success',
                discovery_working: this.results.discovery.status === 'success',
                websocket_connected: this.results.websocket_connection.status === 'success',
                get_models_working: this.results.get_models_test.status === 'success',
                select_model_working: this.results.select_model_test.status === 'success',
                chat_streaming_working: this.results.chat_test.status === 'success',
                backend_port: this.results.discovery.port
            }
        };

        const allPassed = report.summary.schema_loaded && 
                         report.summary.discovery_working && 
                         report.summary.websocket_connected && 
                         report.summary.get_models_working && 
                         report.summary.select_model_working && 
                         report.summary.chat_streaming_working;
        
        console.log('\n📋 VALIDATION REPORT');
        console.log('='.repeat(50));
        console.log(`Phase: ${report.phase} - ${report.description}`);
        console.log(`Timestamp: ${report.timestamp}`);
        console.log('');
        
        console.log('Results:');
        console.log(`  Schema Load: ${this.results.schema_load.status === 'success' ? '✅' : '❌'} ${this.results.schema_load.status}`);
        if (this.results.schema_load.error) {
            console.log(`    Error: ${this.results.schema_load.error}`);
        }
        
        console.log(`  Discovery Test: ${this.results.discovery.status === 'success' ? '✅' : '❌'} ${this.results.discovery.status}`);
        if (this.results.discovery.error) {
            console.log(`    Error: ${this.results.discovery.error}`);
        }
        if (this.results.discovery.port) {
            console.log(`    Backend Port: ${this.results.discovery.port}`);
        }
        
        console.log(`  WebSocket Connection: ${this.results.websocket_connection.status === 'success' ? '✅' : '❌'} ${this.results.websocket_connection.status}`);
        if (this.results.websocket_connection.error) {
            console.log(`    Error: ${this.results.websocket_connection.error}`);
        }
        
        console.log(`  get_models Test: ${this.results.get_models_test.status === 'success' ? '✅' : '❌'} ${this.results.get_models_test.status}`);
        if (this.results.get_models_test.error) {
            console.log(`    Error: ${this.results.get_models_test.error}`);
        }
        if (this.results.get_models_test.response && this.results.get_models_test.response.models) {
            console.log(`    Models Found: ${this.results.get_models_test.response.models.length}`);
        }
        
        console.log(`  select_model Test: ${this.results.select_model_test.status === 'success' ? '✅' : '❌'} ${this.results.select_model_test.status}`);
        if (this.results.select_model_test.error) {
            console.log(`    Error: ${this.results.select_model_test.error}`);
        }
        
        console.log(`  Chat Streaming Test: ${this.results.chat_test.status === 'success' ? '✅' : '❌'} ${this.results.chat_test.status}`);
        if (this.results.chat_test.error) {
            console.log(`    Error: ${this.results.chat_test.error}`);
        }
        if (this.results.chat_test.tokens_received > 0) {
            console.log(`    Tokens Received: ${this.results.chat_test.tokens_received}`);
        }
        if (this.results.chat_test.generation_complete) {
            console.log(`    Generation Complete: ✅ Received`);
        }
        
        
        console.log('');
        console.log(`Overall Status: ${allPassed ? '✅ PASS' : '❌ FAIL'}`);
        console.log('='.repeat(50));

        return { report, passed: allPassed };
    }

    /**
     * Main validation entry point
     */
    async validate(schemaUrl) {
        console.log('🚀 Starting Shimmy Theme Validator (Phase 3.1-3.4)');
        console.log('');

        try {
            // Phase 3.1: Load schema and compile validator
            await this.loadSchema(schemaUrl);
            
            // Phase 3.2: Test discovery
            await this.testDiscovery();
            
            // Phase 3.3: Test WebSocket connection
            await this.testWebSocketConnection();
            
            // Phase 3.4: Test message protocols
            await this.testMessageProtocols();
            
        } catch (error) {
            console.error('💥 Validation failed with error:', error.message);
        }

        // Generate and return report
        const { report, passed } = this.generateReport();
        return { report, passed };
    }
}

// CLI execution
if (require.main === module) {
    async function main() {
        const validator = new ShimmyValidator();

        // CLI: accept --schema-url <url> or -s <url>
        let schemaUrl = null;
        const longIdx = process.argv.indexOf('--schema-url');
        const shortIdx = process.argv.indexOf('-s');
        if (longIdx !== -1 && process.argv.length > longIdx + 1) schemaUrl = process.argv[longIdx + 1];
        else if (shortIdx !== -1 && process.argv.length > shortIdx + 1) schemaUrl = process.argv[shortIdx + 1];
        else schemaUrl = process.env.SHIMMY_SCHEMA_URL || null;

        const { report, passed } = await validator.validate(schemaUrl);

        // Write verify-report file if requested
        try {
            const outPath = process.env.VERIFY_REPORT_FILE || process.env.REPORT_FILE || 'verify-report.json';
            await fs.promises.writeFile(outPath, JSON.stringify(report, null, 2), 'utf8');
            console.log('WROTE verify report to: ' + outPath);
        } catch (err) {
            console.error('Failed to write verify report file:', err.message);
        }

        process.exit(passed ? 0 : 1);
    }

    main().catch(error => {
        console.error('Validator crashed:', error);
        // Attempt to write a crash report for debugging
        try {
            const outPath = process.env.VERIFY_REPORT_FILE || process.env.REPORT_FILE || 'verify-report.json';
            const crashReport = { timestamp: new Date().toISOString(), phase: 'validator_crash', error: String(error) };
            fs.promises.writeFile(outPath, JSON.stringify(crashReport, null, 2), 'utf8').catch(() => {});
        } catch (_) {}
        process.exit(1);
    });
}

module.exports = ShimmyValidator;