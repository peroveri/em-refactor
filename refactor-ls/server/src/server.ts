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

import { generateJsonCodeActions, canExecuteGenerateTestCommand, handleExecuteGenerateTestCommand } from "./create-test-file";

import {
	listActionsForRange,
} from './rust-refactor/refactoring-mappings';

import config from './config';
import { handleExecuteRefactoringCommand } from './rust-refactor/handleExecuteRefactoringCommand';
import { showTypeOrMacroExpansion } from './rust-hover/showTypeOrMacroExpansion';

// Create a connection for the server. The connection uses Node's IPC as a transport.
// Also include all preview / proposed LSP features.
let connection = createConnection(ProposedFeatures.all);

// Create a simple text document manager. The text document manager
// supports full document sync only
let documents: TextDocuments = new TextDocuments();

let hasConfigurationCapability: boolean = false;
let hasWorkspaceFolderCapability: boolean = false;
let hasDiagnosticRelatedInformationCapability: boolean = false;

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
	hasDiagnosticRelatedInformationCapability = !!(
		capabilities.textDocument &&
		capabilities.textDocument.publishDiagnostics &&
		capabilities.textDocument.publishDiagnostics.relatedInformation
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

// The example settings
// interface ExampleSettings {
// 	maxNumberOfProblems: number;
// }

// The global settings, used when the `workspace/configuration` request is not supported by the client.
// Please note that this is not the case when using this server with the client provided in this example
// but could happen with other clients.
// const defaultSettings: ExampleSettings = { maxNumberOfProblems: 1000 };
// let globalSettings: ExampleSettings = defaultSettings;

// Cache the settings of all open documents
// let documentSettings: Map<string, Thenable<ExampleSettings>> = new Map();

// connection.onDidChangeConfiguration(change => {
// 	if (hasConfigurationCapability) {
// 		// Reset all cached document settings
// 		documentSettings.clear();
// 	} else {
// 		globalSettings = <ExampleSettings>(
// 			(change.settings.languageServerExample || defaultSettings)
// 		);
// 	}

// 	// Revalidate all open text documents
// 	documents.all().forEach(validateTextDocument);
// });

// function getDocumentSettings(resource: string): Thenable<ExampleSettings> {
// 	if (!hasConfigurationCapability) {
// 		return Promise.resolve(globalSettings);
// 	}
// 	let result = documentSettings.get(resource);
// 	if (!result) {
// 		result = connection.workspace.getConfiguration({
// 			scopeUri: resource,
// 			section: 'languageServerExample'
// 		});
// 		documentSettings.set(resource, result);
// 	}
// 	return result;
// }

// Only keep settings for open documents
// documents.onDidClose(e => {
// 	documentSettings.delete(e.document.uri);
// });

connection.onCodeAction(handleCodeAction);
connection.onExecuteCommand(handleExecuteCommand);

function handleCodeAction(params: CodeActionParams): Promise<(Command | CodeAction)[]> {

	const doc = documents.get(params.textDocument.uri);
	if (doc === undefined) {
		return Promise.resolve([]);
	}
	let result: (Command | CodeAction)[] = [];
	if(config.showGenerateTestFileCodeActions) {
		result = result.concat(listActionsForRange(doc, params.range));
	}
	result = result.concat(generateJsonCodeActions(refactorings, doc, params));
	return Promise.resolve(result);
}

async function handleExecuteCommand(params: ExecuteCommandParams): Promise<ApplyWorkspaceEditParams | void> {
	if (canExecuteGenerateTestCommand(params)) {
		const edits = await handleExecuteGenerateTestCommand(params);
		for (const edit of edits) {
			await connection.workspace.applyEdit(edit);
		}
		return Promise.resolve();
	}
	return handleExecuteRefactoringCommand(params, connection, documents);
}

connection.onHover(handleOnHover);

async function handleOnHover(params: TextDocumentPositionParams): Promise<Hover | null> {
	if (!config.showTypeOnHover) {
		return Promise.resolve(null);
	}
	return showTypeOrMacroExpansion(params, connection, documents);
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
