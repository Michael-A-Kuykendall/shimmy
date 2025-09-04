import * as vscode from 'vscode';
import { exec, spawn, ChildProcess } from 'child_process';
import * as path from 'path';

let shimmyProcess: ChildProcess | null = null;
let statusBarItem: vscode.StatusBarItem;

export function activate(context: vscode.ExtensionContext) {
    console.log('Shimmy extension is now active!');

    // Create status bar item
    statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
    statusBarItem.text = "$(circle-outline) Shimmy";
    statusBarItem.tooltip = "Shimmy Server Status";
    statusBarItem.command = 'shimmy.start';
    statusBarItem.show();

    // Register commands
    const startCommand = vscode.commands.registerCommand('shimmy.start', startShimmyServer);
    const stopCommand = vscode.commands.registerCommand('shimmy.stop', stopShimmyServer);
    const serveCommand = vscode.commands.registerCommand('shimmy.serve', serveModelWithShimmy);

    context.subscriptions.push(startCommand, stopCommand, serveCommand, statusBarItem);

    // Auto-start if workspace has models
    checkForModelsAndPromptStart();
}

async function startShimmyServer() {
    if (shimmyProcess) {
        vscode.window.showInformationMessage('Shimmy server is already running');
        return;
    }

    const config = vscode.workspace.getConfiguration('shimmy');
    const binaryPath = config.get<string>('binaryPath', 'shimmy');
    const useAutoPort = config.get<boolean>('autoPort', true);
    
    try {
        const args = ['serve'];
        if (!useAutoPort) {
            const manualPort = config.get<number>('manualPort', 11435);
            args.push('--bind', `127.0.0.1:${manualPort}`);
        } else {
            args.push('--bind', 'auto');
        }

        shimmyProcess = spawn(binaryPath, args);
        
        shimmyProcess.stdout?.on('data', (data) => {
            const output = data.toString();
            console.log('Shimmy:', output);
            
            // Extract port from "🚀 Starting Shimmy server on 127.0.0.1:PORT"
            const portMatch = output.match(/127\.0\.0\.1:(\d+)/);
            if (portMatch) {
                const port = portMatch[1];
                updateStatusBar('running', port);
                vscode.window.showInformationMessage(`Shimmy server started on port ${port}`);
            }
        });

        shimmyProcess.stderr?.on('data', (data) => {
            console.error('Shimmy error:', data.toString());
        });

        shimmyProcess.on('exit', (code) => {
            shimmyProcess = null;
            updateStatusBar('stopped');
            if (code !== 0) {
                vscode.window.showErrorMessage(`Shimmy server exited with code ${code}`);
            } else {
                vscode.window.showInformationMessage('Shimmy server stopped');
            }
        });

        updateStatusBar('starting');
        
    } catch (error) {
        vscode.window.showErrorMessage(`Failed to start Shimmy: ${error}`);
        updateStatusBar('error');
    }
}

async function stopShimmyServer() {
    if (!shimmyProcess) {
        vscode.window.showWarningMessage('Shimmy server is not running');
        return;
    }

    shimmyProcess.kill('SIGTERM');
    shimmyProcess = null;
    updateStatusBar('stopped');
}

async function serveModelWithShimmy(uri?: vscode.Uri) {
    if (!uri) {
        const files = await vscode.window.showOpenDialog({
            canSelectFiles: true,
            canSelectFolders: false,
            canSelectMany: false,
            filters: {
                'Model Files': ['gguf', 'safetensors']
            }
        });
        
        if (!files || files.length === 0) {
            return;
        }
        
        uri = files[0];
    }

    const modelPath = uri.fsPath;
    const modelName = path.basename(modelPath, path.extname(modelPath));
    
    // Set environment variable for model
    process.env.SHIMMY_BASE_GGUF = modelPath;
    
    vscode.window.showInformationMessage(`Serving model: ${modelName}`);
    await startShimmyServer();
}

function updateStatusBar(status: 'starting' | 'running' | 'stopped' | 'error', port?: string) {
    switch (status) {
        case 'starting':
            statusBarItem.text = "$(sync~spin) Shimmy Starting";
            statusBarItem.backgroundColor = new vscode.ThemeColor('statusBarItem.warningBackground');
            break;
        case 'running':
            statusBarItem.text = `$(check) Shimmy :${port || '?'}`;
            statusBarItem.backgroundColor = new vscode.ThemeColor('statusBarItem.prominentBackground');
            statusBarItem.command = 'shimmy.stop';
            break;
        case 'stopped':
            statusBarItem.text = "$(circle-outline) Shimmy";
            statusBarItem.backgroundColor = undefined;
            statusBarItem.command = 'shimmy.start';
            break;
        case 'error':
            statusBarItem.text = "$(error) Shimmy";
            statusBarItem.backgroundColor = new vscode.ThemeColor('statusBarItem.errorBackground');
            statusBarItem.command = 'shimmy.start';
            break;
    }
}

async function checkForModelsAndPromptStart() {
    const workspaceFolders = vscode.workspace.workspaceFolders;
    if (!workspaceFolders) {
        return;
    }

    // Look for model files in workspace
    const modelFiles = await vscode.workspace.findFiles('**/*.{gguf,safetensors}', '**/node_modules/**', 5);
    
    if (modelFiles.length > 0) {
        const action = await vscode.window.showInformationMessage(
            `Found ${modelFiles.length} model file(s). Start Shimmy server?`,
            'Start Server',
            'Not Now'
        );
        
        if (action === 'Start Server') {
            await startShimmyServer();
        }
    }
}

export function deactivate() {
    if (shimmyProcess) {
        shimmyProcess.kill('SIGTERM');
    }
}