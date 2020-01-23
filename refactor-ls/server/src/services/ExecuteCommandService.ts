import { singleton, inject } from "tsyringe";
import { Connection, ExecuteCommandParams, ApplyWorkspaceEditParams } from 'vscode-languageserver';
import { canExecuteGenerateTestCommand, handleExecuteGenerateTestCommand, RefactorArgs, getFileRelativePath, convertToCmd, mapRefactorResultToWorkspaceEdit } from "../modules";
import { SettingsService } from "./SettingsService";
import { NotificationService } from "./NotificationService";
import * as shell from 'shelljs';

@singleton()
export class ExecuteCommandService {
    constructor(
        @inject("Connection") private connection: Connection,
        @inject(SettingsService) private settings: SettingsService,
        @inject(NotificationService) private notificationService: NotificationService) {
    }

    handleExecuteCommand = async (params: ExecuteCommandParams): Promise<ApplyWorkspaceEditParams | void | any> => {
        let settings = await this.settings.getSettings();
        if (settings.isGenerateTestFilesEnabled && canExecuteGenerateTestCommand(params)) {
            const edits = await handleExecuteGenerateTestCommand(params);
            for (const edit of edits) {
                await this.connection.workspace.applyEdit(edit);
            }
            return Promise.resolve();
        }
        return this.handleExecuteRefactoringCommand(params, settings.refactoringBinaryPath);
    };


    async handleExecuteRefactoringCommand(params: ExecuteCommandParams, binaryPath: string): Promise<ApplyWorkspaceEditParams | void> {

        if (params.arguments && params.arguments[0]) {
            let arg = params.arguments[0] as RefactorArgs;
            if (!isValidArgs(arg))
                return Promise.reject(`invalid args: ${JSON.stringify(params.arguments)}`);
            let workspaceFolders = await this.connection.workspace.getWorkspaceFolders();
            let relativeFilePath = getFileRelativePath(arg.file, workspaceFolders);
            if (relativeFilePath === undefined || workspaceFolders === null)
                return Promise.reject("unknown file path");
            let workspace_uri = workspaceFolders[0].uri;
            let cmd = convertToCmd(relativeFilePath, arg.refactoring, arg.selection, arg.refactoring === 'extract-method' ? 'foo' : null, arg.unsafe, binaryPath);
            if (cmd instanceof Error) {
                this.notificationService.sendErrorNotification(cmd.message);
                return Promise.reject(cmd.message);
            }
            /* https://github.com/shelljs/shelljs/wiki/Electron-compatibility */
            if (shell.config.execPath === null) {
                shell.config.execPath = shell.which('node').toString();
            }
            let result = shell.exec(cmd);
            if (result.code === 0) {
                let edits = mapRefactorResultToWorkspaceEdit(arg, result.stdout, workspace_uri);

                await this.connection.workspace.applyEdit(edits);

                this.notificationService.sendInfoNotification(`Applied: ${arg.refactoring}`);

                return Promise.resolve();
            }
            else {
                this.notificationService.sendErrorNotification(`Refactoring failed. \nstderr: ${result.stderr}\nstdout: ${result.stdout}`);

                this.notificationService.logError(`Got error code: ${result.code}`);
                return Promise.reject("refactoring failed");
            }
        }
        return Promise.reject("empty argument list");
    }

}

const isValidArgs = (args: RefactorArgs) => {
	return args && args.file;
}
