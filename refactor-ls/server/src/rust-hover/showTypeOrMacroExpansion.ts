import { TextDocumentPositionParams, Hover, MarkedString, Connection, TextDocuments } from 'vscode-languageserver';
import { getFileRelativePath, convertToCmdProvideType } from '../rust-refactor/refactoring-mappings';

let shell = require('shelljs');

export async function showTypeOrMacroExpansion(params: TextDocumentPositionParams, connection: Connection, documents: TextDocuments): Promise<Hover> {
	let workspaceFolders = await connection.workspace.getWorkspaceFolders();
	let relativeFilePath = getFileRelativePath(params.textDocument.uri, workspaceFolders);
	if (relativeFilePath === undefined || workspaceFolders === null)
		return Promise.reject("unknown file path");
	const doc = documents.get(params.textDocument.uri);
	if (doc === undefined) {
		return Promise.reject();
	}
	let pos = doc.offsetAt(params.position);
	let cmd = convertToCmdProvideType(relativeFilePath, `${pos}:${pos}`);
	/* https://github.com/shelljs/shelljs/wiki/Electron-compatibility */
	if (shell.config.execPath === null) {
		shell.config.execPath = shell.which('node').toString();
	}
	console.log(`cmd: ${cmd}`);
	let result = shell.exec(cmd);
	if (result.code === 0) {
		let res = JSON.parse(result.stdout) as Array<{
			type: string;
		}>;
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
		} as Hover);
	}
	return Promise.reject("refactoring failed");
}
