import { WebSocket } from 'ws';
import * as fs from 'fs';
import * as path from 'path';

class ToolTester {
    constructor(wsUrl = 'ws://127.0.0.1:61455/ws/console') {
        this.wsUrl = wsUrl;
        this.ws = null;
        this.messageQueue = [];
        this.isConnected = false;
        this.testResults = [];
    }

    async connect() {
        return new Promise((resolve, reject) => {
            console.log('🔌 Connecting to WebSocket...');
            this.ws = new WebSocket(this.wsUrl);

            this.ws.on('open', () => {
                console.log('✅ Connected to Shimmy WebSocket');
                this.isConnected = true;
                resolve();
            });

            this.ws.on('message', (data) => {
                try {
                    const message = JSON.parse(data.toString());
                    this.handleMessage(message);
                } catch (e) {
                    console.log('📨 Raw message:', data.toString());
                }
            });

            this.ws.on('error', (error) => {
                console.error('❌ WebSocket error:', error);
                reject(error);
            });

            this.ws.on('close', () => {
                console.log('🔌 WebSocket closed');
                this.isConnected = false;
            });
        });
    }

    handleMessage(message) {
        console.log('📨 Received:', JSON.stringify(message, null, 2));

        if (message.type === 'tool_call') {
            this.handleToolCall(message);
        } else if (message.type === 'response') {
            this.handleResponse(message);
        } else if (message.type === 'error') {
            this.handleError(message);
        }
    }

    handleToolCall(message) {
        console.log('🔧 Tool call received:', message.tool_call);
        // Auto-approve tool calls for testing
        this.sendMessage({
            type: 'tool_approval',
            tool_call_id: message.tool_call.id,
            approved: true
        });
    }

    handleResponse(message) {
        console.log('💬 AI Response:', message.content);
        this.lastResponse = message;
    }

    handleError(message) {
        console.error('❌ Error:', message.error);
    }

    sendMessage(message) {
        if (!this.isConnected) {
            throw new Error('WebSocket not connected');
        }

        console.log('📤 Sending:', JSON.stringify(message, null, 2));
        this.ws.send(JSON.stringify(message));
    }

    async sendChatMessage(content, waitForResponse = true) {
        const messageId = Date.now().toString();

        this.sendMessage({
            type: 'chat',
            id: messageId,
            content: content
        });

        if (waitForResponse) {
            return this.waitForResponse(messageId);
        }
    }

    async waitForResponse(messageId, timeout = 300000) { // 5 minutes default
        return new Promise((resolve, reject) => {
            const startTime = Date.now();

            const checkResponse = () => {
                if (this.lastResponse && this.lastResponse.id === messageId) {
                    resolve(this.lastResponse);
                    return;
                }

                if (Date.now() - startTime > timeout) {
                    reject(new Error(`Timeout waiting for response to message ${messageId}`));
                    return;
                }

                setTimeout(checkResponse, 1000);
            };

            checkResponse();
        });
    }

    async testToolAwareness() {
        console.log('\n🧠 Testing Tool Awareness...');

        const response = await this.sendChatMessage(
            'What tools do you have access to? Please list them clearly.'
        );

        this.testResults.push({
            test: 'tool_awareness',
            success: response.content.includes('tool') || response.content.includes('Tool'),
            response: response.content,
            timestamp: new Date().toISOString()
        });

        return response;
    }

    async testSpecificTool(toolName, description, testPrompt) {
        console.log(`\n🔧 Testing ${toolName}...`);

        try {
            const response = await this.sendChatMessage(testPrompt);

            const success = this.evaluateToolTest(toolName, response);

            this.testResults.push({
                test: `tool_${toolName.toLowerCase().replace(/\s+/g, '_')}`,
                tool: toolName,
                description: description,
                success: success,
                response: response.content,
                timestamp: new Date().toISOString()
            });

            return { success, response };
        } catch (error) {
            console.error(`❌ Error testing ${toolName}:`, error);
            this.testResults.push({
                test: `tool_${toolName.toLowerCase().replace(/\s+/g, '_')}`,
                tool: toolName,
                description: description,
                success: false,
                error: error.message,
                timestamp: new Date().toISOString()
            });
            return { success: false, error };
        }
    }

    evaluateToolTest(toolName, response) {
        // Basic evaluation - check if the response mentions the tool or shows understanding
        const content = response.content.toLowerCase();
        const toolKey = toolName.toLowerCase().replace(/\s+/g, '');

        // Check for tool mentions, execution attempts, or understanding
        return content.includes(toolKey) ||
               content.includes('tool') ||
               content.includes('execut') ||
               content.includes('run') ||
               content.includes('file') ||
               content.includes('command');
    }

    async runComprehensiveToolTest() {
        console.log('🚀 Starting Comprehensive Tool Testing Suite...');

        // Test tool awareness first
        await this.testToolAwareness();

        // Define all tools to test with safe, non-destructive prompts
        const toolsToTest = [
            {
                name: 'Read File Tool',
                description: 'Reads content from files',
                prompt: 'Can you help me read the content of a file? Try reading the README.md file in the current directory.'
            },
            {
                name: 'Write File Tool',
                description: 'Writes content to files',
                prompt: 'Can you create a simple test file called "tool_test.txt" with the content "Hello from tool testing!"?'
            },
            {
                name: 'List Files Tool',
                description: 'Lists files in directories',
                prompt: 'What files are in the current directory? Can you list them for me?'
            },
            {
                name: 'Search Files Tool',
                description: 'Searches for text in files',
                prompt: 'Can you search for the word "tool" in the codebase files?'
            },
            {
                name: 'Run Command Tool',
                description: 'Executes shell commands',
                prompt: 'Can you run a simple command like "echo Hello World" and show me the output?'
            },
            {
                name: 'Git Status Tool',
                description: 'Shows git repository status',
                prompt: 'What is the current git status of this repository?'
            },
            {
                name: 'Git Diff Tool',
                description: 'Shows git differences',
                prompt: 'Can you show me the git diff for any uncommitted changes?'
            },
            {
                name: 'Git Log Tool',
                description: 'Shows git commit history',
                prompt: 'Can you show me the recent git commit history?'
            },
            {
                name: 'Project Analysis Tool',
                description: 'Analyzes project structure',
                prompt: 'Can you analyze this project and tell me what type of project it is?'
            },
            {
                name: 'Syntax Check Tool',
                description: 'Checks code syntax',
                prompt: 'Can you check the syntax of the main Rust files in this project?'
            },
            {
                name: 'Build Project Tool',
                description: 'Builds the project',
                prompt: 'Can you check if this project builds successfully?'
            },
            {
                name: 'Run Tests Tool',
                description: 'Runs project tests',
                prompt: 'Can you run the tests for this project?'
            },
            {
                name: 'Explain Command Tool',
                description: 'Explains shell commands',
                prompt: 'Can you explain what the "ls" command does?'
            },
            {
                name: 'Get Help Tool',
                description: 'Provides help information',
                prompt: 'Can you help me understand how to use git?'
            },
            {
                name: 'System Metrics Tool',
                description: 'Shows system information',
                prompt: 'Can you show me some basic system metrics like CPU or memory usage?'
            }
        ];

        // Test each tool
        for (const tool of toolsToTest) {
            await this.testSpecificTool(tool.name, tool.description, tool.prompt);
            // Brief pause between tests
            await new Promise(resolve => setTimeout(resolve, 2000));
        }

        // Generate test report
        this.generateTestReport();
    }

    generateTestReport() {
        console.log('\n📊 Tool Testing Report');
        console.log('='.repeat(50));

        const successful = this.testResults.filter(r => r.success).length;
        const total = this.testResults.length;

        console.log(`✅ Successful tests: ${successful}/${total}`);
        console.log(`❌ Failed tests: ${total - successful}/${total}`);
        console.log(`📈 Success rate: ${((successful / total) * 100).toFixed(1)}%`);

        console.log('\n📋 Detailed Results:');
        this.testResults.forEach(result => {
            const status = result.success ? '✅' : '❌';
            console.log(`${status} ${result.test}: ${result.tool || 'N/A'}`);
            if (!result.success && result.error) {
                console.log(`   Error: ${result.error}`);
            }
        });

        // Save detailed report
        const reportPath = path.join(process.cwd(), 'tool-test-report.json');
        fs.writeFileSync(reportPath, JSON.stringify({
            summary: {
                total_tests: total,
                successful_tests: successful,
                failed_tests: total - successful,
                success_rate: (successful / total) * 100
            },
            results: this.testResults,
            timestamp: new Date().toISOString()
        }, null, 2));

        console.log(`\n💾 Detailed report saved to: ${reportPath}`);
    }

    async close() {
        if (this.ws) {
            this.ws.close();
        }
    }
}

// Main execution
async function main() {
    const tester = new ToolTester();

    try {
        await tester.connect();
        await tester.runComprehensiveToolTest();
    } catch (error) {
        console.error('❌ Test suite failed:', error);
    } finally {
        await tester.close();
    }
}

export default ToolTester;

// Allow direct execution
if (import.meta.url === `file://${process.argv[1]}`) {
    main().catch(console.error);
}