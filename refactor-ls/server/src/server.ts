/* --------------------------------------------------------------------------------------------
 * Copyright (c) Microsoft Corporation. All rights reserved.
 * Licensed under the MIT License. See License.txt in the project root for license information.
 * ------------------------------------------------------------------------------------------ */

import {
	createConnection,
	TextDocuments,
	// TextDocument,
	// Diagnostic,
	// DiagnosticSeverity,
	ProposedFeatures,
	InitializeParams,
	DidChangeConfigurationNotification,
	ShowMessageNotification,
	// CompletionItem,
	// CompletionItemKind,
	// TextDocumentPositionParams,
	CodeActionKind,
	CodeAction,
	Command,
	CodeActionParams,
	ExecuteCommandParams,
	MessageType,
	TextDocumentEdit,
	Range,
	TextDocument,
	Position,
	TextEdit,
	WorkspaceFolder
} from 'vscode-languageserver';
import { uriToFilePath } from 'vscode-languageserver/lib/files';

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
			}
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

class ByteRange {
	constructor(public start: Number, public end: Number) { }
	isRange = () => this.start >= 0 && this.end >= 0;
	isEmpty = () => this.start === this.end;
	toString() {
		return `${this.start}:${this.end}`;
	}
	static Empty = () => new ByteRange(0, 0);
	static Null = () => new ByteRange(-1, -1);
	static fromRange(range: Range, doc: TextDocument): ByteRange {
		const hasSelection = range && range.start && range.end;
		if (!hasSelection || doc === undefined) return this.Null();

		if (range.start.character === range.end.character && range.start.line === range.end.line) return this.Empty();
		return new ByteRange(doc.offsetAt(range.start), doc.offsetAt(range.end))
	}
}



interface RefactorArgs {
	file: string;
	version: number;
	refactoring: string;
	selection: string;
}

/**
 * TODO: Query the refactoring tool for possible refactorings at a given range.
 */
function listActionsForRange(doc: TextDocument, range: ByteRange): (Command | CodeAction)[] {
	return [
		{
			title: `Refactor - Extract block: ${range.toString()}`,
			command: {
				title: 'refactor',
				command: CodeActionKind.RefactorExtract + '.function',
				arguments: [{ file: doc.uri, version: doc.version, selection: range.toString(), refactoring: 'extract-block' }]
			},
			kind: CodeActionKind.RefactorExtract + '.function'
		},
		{
			title: `Refactor - Extract method: ${range.toString()}`,
			command: {
				title: 'refactor',
				command: CodeActionKind.RefactorExtract + '.function',
				arguments: [{ file: doc.uri, version: doc.version, selection: range.toString(), refactoring: 'extract-method' }]
			},
			kind: CodeActionKind.RefactorExtract + '.function'
		}
	];
}

function handleCodeAction(params: CodeActionParams): Promise<(Command | CodeAction)[]> {

	const doc = documents.get(params.textDocument.uri);
	if (doc === undefined) {
		return Promise.resolve([]);
	}
	const range = ByteRange.fromRange(params.range, doc);

	if (!range.isRange() || range.isEmpty()) {
		return Promise.resolve([]);
	}

	return Promise.resolve(listActionsForRange(doc, range));
}

const isValidArgs = (args: RefactorArgs) => {
	return args && args.file;
}

const mapResultToWorkspaceEdit = (arg: RefactorArgs, stdout: string, doc: TextDocument) => {
	let res = JSON.parse(stdout) as [{
		file_name: string;
		start: number;
		end: number;
		replacement: string;
	}];

	let edits = res.map(change => ({
		newText: change.replacement,
		range: {
			start: doc.positionAt(change.start),
			end: doc.positionAt(change.end)
		}
	} as TextEdit));
	let documentChanges: TextDocumentEdit[] = [
		{
			textDocument: { uri: arg.file, version: arg.version },
			edits: edits
		}
	];
	return {
		edit: {
			documentChanges: documentChanges
		},
		label: arg.refactoring
	};
}

const getFileRelativePath = (fileUri: string, workspace: WorkspaceFolder[] | null) => {
	if (workspace === null || workspace.length === 0) return undefined;
	let workspaceUri = workspace[0].uri;
	return getRelativePath(workspaceUri, fileUri);
}

const getRelativePath = (workspaceUri: string, fileUri: string) => {
	if (fileUri.startsWith(workspaceUri)) {
		let sub = fileUri.substring(workspaceUri.length);
		if (sub.startsWith("/")) sub = sub.substring(1);
		return sub;
	}
	return undefined;
}

const convertToCmd = (relativeFilePath: string, refactoring: string, selection: string, new_fn: string | null) => {
	const refactorManifestPath = '/home/perove/dev/github.uio.no/refactor-rust/Cargo.toml'; // TODO: hardcoded path to refactoring project
	const refactorArgs = `--output-changes-as-json --file=${relativeFilePath} --refactoring=${refactoring} --selection=${selection}` + (new_fn === null ? '' : ` --new_function=${new_fn}`);

	const rustcArgs = relativeFilePath;

	return `cargo run --bin my-refactor-driver --manifest-path=${refactorManifestPath} -- ${rustcArgs} -- ${refactorArgs}`;
}

async function handleExecuteCommand(params: ExecuteCommandParams): Promise<void> {
	console.log('handleExecuteCommand', params);
	if (params.arguments && params.arguments[0]) {
		let arg = params.arguments[0] as RefactorArgs;
		if (!isValidArgs(arg)) return Promise.resolve();

		let w = await connection.workspace.getWorkspaceFolders();
		let relativeFilePath = getFileRelativePath(arg.file, w);
		if (relativeFilePath === undefined) return Promise.resolve();

		let cmd = convertToCmd(relativeFilePath, arg.refactoring, arg.selection, arg.refactoring === 'extract-method' ? 'foo' : null);

		let result = shell.exec(cmd);

		if (result.code === 0) {

			const doc = documents.get(arg.file);
			if (doc === undefined) return Promise.resolve();

			connection.workspace.applyEdit(mapResultToWorkspaceEdit(arg, result.stdout, doc));
			connection.sendNotification(ShowMessageNotification.type, {
				message: `Applied: ${arg.refactoring}`, type: MessageType.Info,
			});
		} else {
			connection.sendNotification(ShowMessageNotification.type, {
				message: `Refactoring failed. \nstderr: ${result.stderr}\nstdout: ${result.stdout}`, type: MessageType.Error,
			});
			console.error(`Got error code: ${result.code}`);
			console.error(cmd);
			console.error(result);

		}
	}
	// console.log(params);

	return Promise.resolve();
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
