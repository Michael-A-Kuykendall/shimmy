import * as vscode from 'vscode';
import * as cp from 'child_process';
import * as http from 'http';

let shimmyProcess: cp.ChildProcess | undefined;
let statusBarItem: vscode.StatusBarItem;

export function activate(context: vscode.ExtensionContext) {
    console.log('Shimmy extension is now active!');

    // Create status bar item
    statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
    statusBarItem.command = 'shimmy.startServer';
    statusBarItem.text = '$(circle-slash) Shimmy: Offline';
    statusBarItem.tooltip = 'Click to start Shimmy server';
    statusBarItem.show();
    context.subscriptions.push(statusBarItem);

    // Register commands
    const startCommand = vscode.commands.registerCommand('shimmy.startServer', startServer);
    const stopCommand = vscode.commands.registerCommand('shimmy.stopServer', stopServer);
    const restartCommand = vscode.commands.registerCommand('shimmy.restartServer', restartServer);
    const generateCommand = vscode.commands.registerCommand('shimmy.generateCode', generateCode);
    const generateFromSelectionCommand = vscode.commands.registerCommand('shimmy.generateFromSelection', generateFromSelection);

    context.subscriptions.push(startCommand, stopCommand, restartCommand, generateCommand, generateFromSelectionCommand);

    // Auto-start if configured
    const config = vscode.workspace.getConfiguration('shimmy');
    if (config.get<boolean>('autoStart')) {
        startServer();
    }

    // Check if server is already running
    checkServerStatus();
}

export function deactivate() {
    if (shimmyProcess) {
        shimmyProcess.kill();
    }
}

async function startServer() {
    if (shimmyProcess) {
        vscode.window.showWarningMessage('Shimmy server is already running');
        return;
    }

    const config = vscode.workspace.getConfiguration('shimmy');
    const binaryPath = config.get<string>('binaryPath') || 'shimmy';
    const modelPath = config.get<string>('modelPath');

    try {
        const env = { ...process.env };
        if (modelPath) {
            env.SHIMMY_BASE_GGUF = modelPath;
        }

        statusBarItem.text = '$(loading~spin) Shimmy: Starting...';
        statusBarItem.tooltip = 'Starting Shimmy server';

        shimmyProcess = cp.spawn(binaryPath, ['serve'], {
            env,
            stdio: 'pipe'
        });

        shimmyProcess.stdout?.on('data', (data) => {
            console.log(`Shimmy stdout: ${data}`);
        });

        shimmyProcess.stderr?.on('data', (data) => {
            console.error(`Shimmy stderr: ${data}`);
        });

        shimmyProcess.on('close', (code) => {
            console.log(`Shimmy process exited with code ${code}`);
            shimmyProcess = undefined;
            statusBarItem.text = '$(circle-slash) Shimmy: Offline';
            statusBarItem.tooltip = 'Click to start Shimmy server';
            statusBarItem.command = 'shimmy.startServer';
        });

        shimmyProcess.on('error', (error) => {
            console.error(`Failed to start Shimmy: ${error}`);
            vscode.window.showErrorMessage(`Failed to start Shimmy: ${error.message}`);
            shimmyProcess = undefined;
            statusBarItem.text = '$(circle-slash) Shimmy: Error';
            statusBarItem.tooltip = 'Failed to start server. Check configuration.';
            statusBarItem.command = 'shimmy.startServer';
        });

        // Wait a moment then check if server is responsive
        setTimeout(checkServerStatus, 2000);

    } catch (error) {
        vscode.window.showErrorMessage(`Failed to start Shimmy server: ${error}`);
        statusBarItem.text = '$(circle-slash) Shimmy: Error';
        statusBarItem.tooltip = 'Failed to start server';
    }
}

async function stopServer() {
    if (shimmyProcess) {
        shimmyProcess.kill('SIGTERM');
        shimmyProcess = undefined;
        statusBarItem.text = '$(circle-slash) Shimmy: Offline';
        statusBarItem.tooltip = 'Click to start Shimmy server';
        statusBarItem.command = 'shimmy.startServer';
        vscode.window.showInformationMessage('Shimmy server stopped');
    } else {
        vscode.window.showWarningMessage('Shimmy server is not running');
    }
}

async function restartServer() {
    if (shimmyProcess) {
        await stopServer();
        // Wait a moment before restarting
        setTimeout(startServer, 1000);
    } else {
        await startServer();
    }
}

async function checkServerStatus() {
    const config = vscode.workspace.getConfiguration('shimmy');
    const serverUrl = config.get<string>('serverUrl') || 'http://localhost:11435';
    
    try {
        const url = new URL('/health', serverUrl);
        
        const options = {
            hostname: url.hostname,
            port: url.port || '80',
            path: url.pathname,
            method: 'GET',
            timeout: 5000
        };

        const req = http.request(options, (res) => {
            if (res.statusCode === 200) {
                statusBarItem.text = '$(check) Shimmy: Online';
                statusBarItem.tooltip = 'Shimmy server is running';
                statusBarItem.command = 'shimmy.stopServer';
            } else {
                statusBarItem.text = '$(circle-slash) Shimmy: Offline';
                statusBarItem.tooltip = 'Click to start Shimmy server';
                statusBarItem.command = 'shimmy.startServer';
            }
        });

        req.on('error', () => {
            statusBarItem.text = '$(circle-slash) Shimmy: Offline';
            statusBarItem.tooltip = 'Click to start Shimmy server';
            statusBarItem.command = 'shimmy.startServer';
        });

        req.on('timeout', () => {
            req.destroy();
            statusBarItem.text = '$(circle-slash) Shimmy: Offline';
            statusBarItem.tooltip = 'Click to start Shimmy server';
            statusBarItem.command = 'shimmy.startServer';
        });

        req.end();
    } catch (error) {
        statusBarItem.text = '$(circle-slash) Shimmy: Offline';
        statusBarItem.tooltip = 'Click to start Shimmy server';
        statusBarItem.command = 'shimmy.startServer';
    }
}

async function generateCode() {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        vscode.window.showErrorMessage('No active editor');
        return;
    }

    const prompt = await vscode.window.showInputBox({
        prompt: 'Enter your code generation prompt',
        placeHolder: 'e.g., "Create a function that calculates fibonacci numbers"'
    });

    if (!prompt) {
        return;
    }

    const config = vscode.workspace.getConfiguration('shimmy');
    const serverUrl = config.get<string>('serverUrl') || 'http://localhost:11435';

    try {
        const response = await makeShimmyRequest(serverUrl, prompt);
        
        // Insert generated code at cursor position
        const position = editor.selection.active;
        editor.edit(editBuilder => {
            editBuilder.insert(position, response);
        });

    } catch (error) {
        vscode.window.showErrorMessage(`Code generation failed: ${error}`);
    }
}

async function generateFromSelection() {
    const editor = vscode.window.activeTextEditor;
    if (!editor || editor.selection.isEmpty) {
        vscode.window.showErrorMessage('No text selected');
        return;
    }

    const selectedText = editor.document.getText(editor.selection);
    const language = editor.document.languageId;
    
    const prompt = `Complete this ${language} code:\n\n${selectedText}`;

    const config = vscode.workspace.getConfiguration('shimmy');
    const serverUrl = config.get<string>('serverUrl') || 'http://localhost:11435';

    try {
        const response = await makeShimmyRequest(serverUrl, prompt);
        
        // Replace selection with generated code
        editor.edit(editBuilder => {
            editBuilder.replace(editor.selection, response);
        });

    } catch (error) {
        vscode.window.showErrorMessage(`Code generation failed: ${error}`);
    }
}

async function makeShimmyRequest(serverUrl: string, prompt: string): Promise<string> {
    return new Promise((resolve, reject) => {
        const data = JSON.stringify({
            model: 'default',
            prompt: prompt,
            max_tokens: 500,
            temperature: 0.1,
            stream: false
        });

        const url = new URL('/api/generate', serverUrl);
        
        const options = {
            hostname: url.hostname,
            port: url.port || '80',
            path: url.pathname,
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Content-Length': Buffer.byteLength(data)
            },
            timeout: 30000
        };

        const req = http.request(options, (res) => {
            let body = '';
            
            res.on('data', (chunk) => {
                body += chunk;
            });

            res.on('end', () => {
                try {
                    const response = JSON.parse(body);
                    if (response.text) {
                        resolve(response.text);
                    } else if (response.error) {
                        reject(new Error(response.error));
                    } else {
                        reject(new Error('Unexpected response format'));
                    }
                } catch (error) {
                    reject(new Error(`Failed to parse response: ${error}`));
                }
            });
        });

        req.on('error', (error) => {
            reject(new Error(`Request failed: ${error.message}`));
        });

        req.on('timeout', () => {
            req.destroy();
            reject(new Error('Request timeout'));
        });

        req.write(data);
        req.end();
    });
}
