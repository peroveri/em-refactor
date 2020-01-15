import { ShowMessageNotification, ExecuteCommandParams, MessageType, ApplyWorkspaceEditParams } from 'vscode-languageserver';
import { convertToCmd, getFileRelativePath, mapRefactorResultToWorkspaceEdit, RefactorArgs } from './refactoring-mappings';
import { isValidArgs, connection, documents } from '../server';

let shell = require('shelljs');

export async function handleExecuteRefactoringCommand(params: ExecuteCommandParams): Promise<ApplyWorkspaceEditParams | void> {
	if (params.arguments && params.arguments[0]) {
		let arg = params.arguments[0] as RefactorArgs;
		if (!isValidArgs(arg))
			return Promise.reject(`invalid args: ${JSON.stringify(params.arguments)}`);
		let workspaceFolders = await connection.workspace.getWorkspaceFolders();
		let relativeFilePath = getFileRelativePath(arg.file, workspaceFolders);
		if (relativeFilePath === undefined || workspaceFolders === null)
			return Promise.reject("unknown file path");
		let workspace_uri = workspaceFolders[0].uri;
		let cmd = convertToCmd(relativeFilePath, arg.refactoring, arg.selection, arg.refactoring === 'extract-method' ? 'foo' : null, arg.unsafe);
		/* https://github.com/shelljs/shelljs/wiki/Electron-compatibility */
		if (shell.config.execPath === null) {
			shell.config.execPath = shell.which('node').toString();
		}
		let result = shell.exec(cmd);
		if (result.code === 0) {
			let edits = mapRefactorResultToWorkspaceEdit(arg, result.stdout, workspace_uri, documents);
			console.log(`stdout: ${result.stdout}`);
			connection.workspace.applyEdit(edits);
			connection.sendNotification(ShowMessageNotification.type, {
				message: `Applied: ${arg.refactoring}`, type: MessageType.Info,
			});
			return Promise.resolve(edits);
		}
		else {
			connection.sendNotification(ShowMessageNotification.type, {
				message: `Refactoring failed. \nstderr: ${result.stderr}\nstdout: ${result.stdout}`, type: MessageType.Error,
			});
			console.error(`Got error code: ${result.code}`);
			console.error(cmd);
			console.error(result);
			return Promise.reject("refactoring failed");
		}
	}
	return Promise.reject("empty argument list");
}
