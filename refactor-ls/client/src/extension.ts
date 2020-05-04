/* --------------------------------------------------------------------------------------------
 * Copyright (c) Microsoft Corporation. All rights reserved.
 * Licensed under the MIT License. See License.txt in the project root for license information.
 * ------------------------------------------------------------------------------------------ */

import * as path from 'path';
import * as vscode from 'vscode';

import {
	LanguageClient,
	LanguageClientOptions,
	ServerOptions,
	TransportKind
} from 'vscode-languageclient';

let client: LanguageClient;
let statusBar: vscode.StatusBarItem;

export function activate(context: vscode.ExtensionContext) {
	// The server is implemented in node
	let serverModule = context.asAbsolutePath(
		path.join('server', 'out', 'server.js')
	);
	// The debug options for the server
	// --inspect=6009: runs the server in Node's Inspector mode so VS Code can attach to the server for debugging
	let debugOptions = { execArgv: ['--nolazy', '--inspect=6009'] };

	// If the extension is launched in debug mode then the debug server options are used
	// Otherwise the run options are used
	let serverOptions: ServerOptions = {
		run: { module: serverModule, transport: TransportKind.ipc },
		debug: {
			module: serverModule,
			transport: TransportKind.ipc,
			options: debugOptions
		}
	};

	// Options to control the language client
	let clientOptions: LanguageClientOptions = {
		// Register the server for plain text documents
		documentSelector: [
			{ scheme: 'file', language: 'rust' },
			{ scheme: 'untitled', language: 'rust' }
		],
		synchronize: {
			// Notify the server about file changes to '.clientrc files contained in the workspace
			fileEvents: [
				vscode.workspace.createFileSystemWatcher('**/.clientrc'),
				vscode.workspace.createFileSystemWatcher('**/.rs'),
			],
		}
	};

	const showCustomCommandsId = 'refactor-tool.showCustomCommands';
	context.subscriptions.push(vscode.commands.registerCommand(showCustomCommandsId, () => {
		vscode.window.showQuickPick(listCustomCommands()).then( async (val) => {
			return await executeCommand(val);
		});
	}));

	statusBar = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
	statusBar.command = showCustomCommandsId;
	context.subscriptions.push(statusBar);
	context.subscriptions.push(vscode.window.onDidChangeTextEditorSelection(updateStatusBarItem));
	
	// Create the language client and start the client.
	client = new LanguageClient(
		'languageServerExample',
		'Language Server Example',
		serverOptions,
		clientOptions
	);

	// Start the client. This will also launch the server
	client.start();
	statusBar.show();
}

function listCustomCommands() {
	return [
		"cargo check --target-dir=./target/refactorings",
		"candidates extract method",
		"candidates box field"
	];
}

function executeCommand(cmd: string) {

	switch (cmd) {
		case 'cargo check --target-dir=./target/refactorings': {
			return vscode.commands.executeCommand("mrefactor.cargo_check");
		}
		case 'candidates extract method': {
			return vscode.commands.executeCommand("mrefactor.candidates", "extract-method");
		}
		case 'candidates box field': {
			return vscode.commands.executeCommand("mrefactor.candidates", "box-field");
		}
		default: {}
	}
}

function updateStatusBarItem() {
	statusBar.show();
	statusBar.text = `Refactor Tool`;
}

export function deactivate(): Thenable<void> | undefined {
	if (!client) {
		return undefined;
	}
	return client.stop();
}
