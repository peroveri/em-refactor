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
	ShowMessageNotification,
	CodeActionKind,
	CodeAction,
	Command,
	CodeActionParams,
	ExecuteCommandParams,
	MessageType,
	ApplyWorkspaceEditParams,
	TextDocumentPositionParams,
	Hover,
	MarkedString,
} from 'vscode-languageserver';

import {
	convertToCmd,
	getFileRelativePath,
	listActionsForRange,
	mapRefactorResultToWorkspaceEdit,
	RefactorArgs,
	convertToCmdProvideType,
} from './refactoring-mappings';

import config from './config';

let shell = require('shelljs');

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
				]
			},
			executeCommandProvider: {
				commands: ['refactor.extract.function']
			},
			hoverProvider: true
		}
	};
});

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

// The content of a text document has changed. This event is emitted
// when the text document first opened or when its content has changed.
// documents.onDidChangeContent(change => {
// validateTextDocument(change.document);
// });
connection.onCodeAction(handleCodeAction);
connection.onExecuteCommand(handleExecuteCommand);

function handleCodeAction(params: CodeActionParams): Promise<(Command | CodeAction)[]> {

	const doc = documents.get(params.textDocument.uri);
	if (doc === undefined) {
		return Promise.resolve([]);
	}

	return Promise.resolve(listActionsForRange(doc, params.range));
}

const isValidArgs = (args: RefactorArgs) => {
	return args && args.file;
}

async function handleExecuteCommand(params: ExecuteCommandParams): Promise<ApplyWorkspaceEditParams> {
	console.log('handleExecuteCommand', params);
	if (params.arguments && params.arguments[0]) {
		let arg = params.arguments[0] as RefactorArgs;
		if (!isValidArgs(arg)) return Promise.reject(`invalid args: ${JSON.stringify(params.arguments)}`);

		let workspaceFolders = await connection.workspace.getWorkspaceFolders();
		let relativeFilePath = getFileRelativePath(arg.file, workspaceFolders);
		if (relativeFilePath === undefined || workspaceFolders === null) return Promise.reject("unknown file path");
		let workspace_uri = workspaceFolders[0].uri;

		let cmd = convertToCmd(relativeFilePath, arg.refactoring, arg.selection, arg.refactoring === 'extract-method' ? 'foo' : null, arg.unsafe);

		/* https://github.com/shelljs/shelljs/wiki/Electron-compatibility */
		if (shell.config.execPath === null) {
			shell.config.execPath = shell.which('node').toString();
		}
		let result = shell.exec(cmd);

		if (result.code === 0) {

			let edits = mapRefactorResultToWorkspaceEdit(arg, result.stdout, workspace_uri, documents);
			console.log(result.stdout);
			connection.workspace.applyEdit(edits);
			connection.sendNotification(ShowMessageNotification.type, {
				message: `Applied: ${arg.refactoring}`, type: MessageType.Info,
			});
			return Promise.resolve(edits);
		} else {
			connection.sendNotification(ShowMessageNotification.type, {
				message: `Refactoring failed. \nstderr: ${result.stderr}\nstdout: ${result.stdout}`, type: MessageType.Error,
			});
			console.error(`Got error code: ${result.code}`);
			console.error(cmd);
			console.error(result);
			return Promise.reject("refactoring failed")
		}
	}
	// console.log(params);

	return Promise.reject("empty arg list");
}

connection.onHover(handleHover);


async function handleHover(params: TextDocumentPositionParams): Promise<Hover | null> {
	console.log(`handle hover`);
	if(!config.showTypeOnHover) {
		return Promise.resolve(null);
	}

	let workspaceFolders = await connection.workspace.getWorkspaceFolders();
	let relativeFilePath = getFileRelativePath(params.textDocument.uri, workspaceFolders);
	if (relativeFilePath === undefined || workspaceFolders === null) return Promise.reject("unknown file path");

	const doc = documents.get(params.textDocument.uri);
	if (doc === undefined) {return Promise.reject();}
	let pos = doc.offsetAt(params.position);
	let cmd = convertToCmdProvideType(relativeFilePath, `${pos}:${pos}`);

	/* https://github.com/shelljs/shelljs/wiki/Electron-compatibility */
	if (shell.config.execPath === null) {
		shell.config.execPath = shell.which('node').toString();
	}
	console.log(`cmd: ${cmd}`);
	let result = shell.exec(cmd);

	if (result.code === 0) {

		let res = JSON.parse(result.stdout) as Array<{type: string}>;
		let content = res && res.length > 0 ? res[0].type : '<empty>';

		content = content.replace(/\n([ \t]+)/g, (match, p1: string) => {
			return '\n' + ' '.repeat((p1.length) / 8);
		});

		return Promise.resolve({
			contents: {
				language: 'rust',
				value: content
			} as MarkedString,
			range: {
				start: params.position,
				end: params.position
			}
		}as Hover );
	}
	return Promise.reject("refactoring failed")
}

// async function validateTextDocument(textDocument: TextDocument): Promise<void> {
// 	// In this simple example we get the settings for every validate run.
// 	let settings = await getDocumentSettings(textDocument.uri);

// 	// The validator creates diagnostics for all uppercase words length 2 and more
// 	let text = textDocument.getText();
// 	let pattern = /\b[A-Z]{2,}\b/g;
// 	let m: RegExpExecArray | null;

// 	let problems = 0;
// 	let diagnostics: Diagnostic[] = [];
// 	while ((m = pattern.exec(text)) && problems < settings.maxNumberOfProblems) {
// 		problems++;
// 		let diagnostic: Diagnostic = {
// 			severity: DiagnosticSeverity.Warning,
// 			range: {
// 				start: textDocument.positionAt(m.index),
// 				end: textDocument.positionAt(m.index + m[0].length)
// 			},
// 			message: `${m[0]} is all uppercase.`,
// 			source: 'ex'
// 		};
// 		if (hasDiagnosticRelatedInformationCapability) {
// 			diagnostic.relatedInformation = [
// 				{
// 					location: {
// 						uri: textDocument.uri,
// 						range: Object.assign({}, diagnostic.range)
// 					},
// 					message: 'Spelling matters'
// 				},
// 				{
// 					location: {
// 						uri: textDocument.uri,
// 						range: Object.assign({}, diagnostic.range)
// 					},
// 					message: 'Particularly for names'
// 				}
// 			];
// 		}
// 		diagnostics.push(diagnostic);
// 	}

// 	// Send the computed diagnostics to VSCode.
// 	connection.sendDiagnostics({ uri: textDocument.uri, diagnostics });
// }

connection.onDidChangeWatchedFiles(_change => {
	// Monitored files have change in VSCode
	connection.console.log('We received an file change event');
});

// This handler provides the initial list of the completion items.
// connection.onCompletion(
// 	(_textDocumentPosition: TextDocumentPositionParams): CompletionItem[] => {
// 		// The pass parameter contains the position of the text document in
// 		// which code complete got requested. For the example we ignore this
// 		// info and always provide the same completion items.
// 		return [
// 			{
// 				label: 'TypeScript',
// 				kind: CompletionItemKind.Text,
// 				data: 1
// 			},
// 			{
// 				label: 'JavaScript',
// 				kind: CompletionItemKind.Text,
// 				data: 2
// 			}
// 		];
// 	}
// );

// This handler resolves additional information for the item selected in
// the completion list.
// connection.onCompletionResolve(
// 	(item: CompletionItem): CompletionItem => {
// 		if (item.data === 1) {
// 			item.detail = 'TypeScript details';
// 			item.documentation = 'TypeScript documentation';
// 		} else if (item.data === 2) {
// 			item.detail = 'JavaScript details';
// 			item.documentation = 'JavaScript documentation';
// 		}
// 		return item;
// 	}
// );

/*
connection.onDidOpenTextDocument((params) => {
	// A text document got opened in VSCode.
	// params.uri uniquely identifies the document. For documents store on disk this is a file URI.
	// params.text the initial full content of the document.
	connection.console.log(`${params.textDocument.uri} opened.`);
});
connection.onDidChangeTextDocument((params) => {
	// The content of a text document did change in VSCode.
	// params.uri uniquely identifies the document.
	// params.contentChanges describe the content changes to the document.
	connection.console.log(`${params.textDocument.uri} changed: ${JSON.stringify(params.contentChanges)}`);
});
connection.onDidCloseTextDocument((params) => {
	// A text document got closed in VSCode.
	// params.uri uniquely identifies the document.
	connection.console.log(`${params.textDocument.uri} closed.`);
});
*/

// Make the text document manager listen on the connection
// for open, change and close text document events
documents.listen(connection);

// Listen on the connection
connection.listen();
