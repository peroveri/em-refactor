/* --------------------------------------------------------------------------------------------
 * Copyright (c) Microsoft Corporation. All rights reserved.
 * Licensed under the MIT License. See License.txt in the project root for license information.
 * ------------------------------------------------------------------------------------------ */

import {
	createConnection,
	TextDocuments,
	ProposedFeatures,
	InitializeParams,
	DidChangeConfigurationNotification,
	CodeActionKind,
	CodeAction,
	Command,
	CodeActionParams,
	ExecuteCommandParams,
	ApplyWorkspaceEditParams,
	TextDocumentPositionParams,
	Hover,
} from 'vscode-languageserver';

import {
	canExecuteGenerateTestCommand,
	generateJsonCodeActions,
	getLSPExtensionSettings,
	handleExecuteGenerateTestCommand,
	handleExecuteRefactoringCommand,
	listActionsForRange,
	showTypeOrMacroExpansion
} from './modules';

// Create a connection for the server. The connection uses Node's IPC as a transport.
// Also include all preview / proposed LSP features.
let connection = createConnection(ProposedFeatures.all);

// Create a simple text document manager. The text document manager
// supports full document sync only
let documents: TextDocuments = new TextDocuments();

let hasConfigurationCapability: boolean = false;
let hasWorkspaceFolderCapability: boolean = false;

connection.onInitialize((params: InitializeParams) => {
	let capabilities = params.capabilities;

	// Does the client support the `workspace/configuration` request?
	// If not, we will fall back using global settings
	hasConfigurationCapability = !!(
		capabilities.workspace && !!capabilities.workspace.configuration
	);
	hasWorkspaceFolderCapability = !!(
		capabilities.workspace && !!capabilities.workspace.workspaceFolders
	);

	return {
		capabilities: {
			textDocumentSync: documents.syncKind,
			// Tell the client that the server supports code completion
			// completionProvider: {
			// 	resolveProvider: true
			// },
			codeActionProvider: { // TODO: code actions literal support
				codeActionKinds: [
					CodeActionKind.RefactorExtract + '.function',
					`${CodeActionKind.Refactor}.generate_test_file`
				]
			},
			executeCommandProvider: {
				commands: [
					'refactor.extract.function',
					`${CodeActionKind.Refactor}.generate_test_file`
				]
			},
			hoverProvider: true
		}
	};
});
const refactorings = [
	"extract-block",
	"box-field"
];
connection.onInitialized(() => {

	if (hasConfigurationCapability) {
		// Register for all configuration changes.
		connection.client.register(DidChangeConfigurationNotification.type, undefined);
	}
	if (hasWorkspaceFolderCapability) {
		connection.workspace.onDidChangeWorkspaceFolders(_event => {
			connection.console.log('Workspace folder change event received.');
		});
	}
	let hasEditCapability = true;
	if (hasEditCapability) {
		// connection.client.register()
	}
});

connection.onCodeAction(handleCodeAction);
connection.onExecuteCommand(handleExecuteCommand);

async function handleCodeAction(params: CodeActionParams): Promise<(Command | CodeAction)[]> {
	let settings = await getLSPExtensionSettings(connection);

	const doc = documents.get(params.textDocument.uri);
	if (doc === undefined) {
		return Promise.resolve([]);
	}
	let result: (Command | CodeAction)[] = [];
	if (settings.isGenerateTestFilesEnabled) {
		result = result.concat(generateJsonCodeActions(refactorings, doc, params));
	}
	result = result.concat(listActionsForRange(doc, params.range));
	return Promise.resolve(result);
}

async function handleExecuteCommand(params: ExecuteCommandParams): Promise<ApplyWorkspaceEditParams | void | any> {
	const settings = await getLSPExtensionSettings(connection);

	if (settings.isGenerateTestFilesEnabled && canExecuteGenerateTestCommand(params)) {
		const edits = await handleExecuteGenerateTestCommand(params);
		for (const edit of edits) {
			await connection.workspace.applyEdit(edit);
		}
		return Promise.resolve();
	}
	return handleExecuteRefactoringCommand(params, connection, settings.refactoringBinaryPath);
}

connection.onHover(handleOnHover);

async function handleOnHover(params: TextDocumentPositionParams): Promise<Hover | null> {
	const settings = await getLSPExtensionSettings(connection);

	if (settings.isHoverEnabled) {
		return showTypeOrMacroExpansion(params, connection, documents, settings.refactoringBinaryPath);
	}
	return Promise.resolve(null);
}

connection.onDidChangeWatchedFiles(_change => {
	// Monitored files have change in VSCode
	connection.console.log('We received an file change event');
});

// Make the text document manager listen on the connection
// for open, change and close text document events
documents.listen(connection);

// Listen on the connection
connection.listen();
