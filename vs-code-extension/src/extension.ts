import * as vscode from 'vscode';

let terminal: vscode.Terminal | undefined;

function getCurrentFilePath(): string | null {
    const editor = vscode.window.activeTextEditor;
    return editor?.document?.fileName || null;
}

function getRustToolPath(): string | undefined {
    return vscode.workspace.getConfiguration().get<string>('http-snap.path');
}

function getEnvironmentPath(): string | undefined {
    return vscode.workspace.getConfiguration().get<string>('http-snap.environment');
}

function getClientOptionsPath(): string | undefined {
    return vscode.workspace.getConfiguration().get<string>('http-snap.client-options');
}

function runCargoCommand(args: string[], environment: string | undefined, clientOptions: string | undefined, cwd: string) {
    const command = 'cargo';
	if (environment != undefined && environment.length > 0) {
		args.push("--environment", environment);
	}
    if (clientOptions != undefined && clientOptions.length > 0) {
        args.push("--client-options", clientOptions);
    }
    const fullCommand = `${command} ${args.join(' ')}`;

	if (!terminal || terminal.exitStatus !== undefined) {
		terminal = vscode.window.createTerminal({
			name: 'Http Snap',
			cwd: cwd
		});
	}

    terminal.show(true);
    terminal.sendText(fullCommand);
}

function registerCommand(commandId: string, argsBuilder: (filePath: string) => string[]) {
    return vscode.commands.registerCommand(commandId, async () => {
		const editor = vscode.window.activeTextEditor;
        const filePath = getCurrentFilePath();
        const toolPath = getRustToolPath();
		const enviromentPath = getEnvironmentPath();
        const clientOptionsPath = getClientOptionsPath();

		if (!editor || !filePath) {
            vscode.window.showErrorMessage('No active file open.');
            return;
        }

        if (!toolPath) {
            vscode.window.showErrorMessage('CLI tool path is not set in settings.');
            return;
        }

		const saved = await editor.document.save();
        if (!saved) {
            vscode.window.showErrorMessage('Failed to save the current file.');
            return;
        }

        const args = argsBuilder(filePath);
        runCargoCommand(args, enviromentPath, clientOptionsPath, toolPath);
    });
}

export function activate(context: vscode.ExtensionContext) {
    context.subscriptions.push(
        registerCommand('http-snap-runner.runTest', (filePath) => ['run', '--', 'test', '--path', filePath]),
        registerCommand('http-snap-runner.runUpdateOverwrite', (filePath) => ['run', '--', 'update', '--path', filePath, '--update-mode', 'overwrite', '--detectors', 'all']),
        registerCommand('http-snap-runner.runUpdateAppend', (filePath) => ['run', '--', 'update', '--path', filePath, '--update-mode', 'append', '--detectors', 'all'])
    );
}

export function deactivate() {}
