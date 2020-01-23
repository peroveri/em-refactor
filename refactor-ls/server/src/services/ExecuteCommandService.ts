import { singleton, inject } from "tsyringe";
import { Connection, ExecuteCommandParams, ApplyWorkspaceEditParams } from 'vscode-languageserver';
import { canExecuteGenerateTestCommand, handleExecuteGenerateTestCommand, RefactorArgs } from "../modules";
import { SettingsService } from "./SettingsService";
import { NotificationService } from "./NotificationService";
import { ShellService } from "./ShellService";
import { WorkspaceService } from "./WorkspaceService";
import { mapRefactorResultToWorkspaceEdit } from "./mappings/workspace-mappings"

@singleton()
export class ExecuteCommandService {
    constructor(
        @inject("Connection") private connection: Connection,
        @inject(SettingsService) private settings: SettingsService,
        @inject(NotificationService) private notificationService: NotificationService,
        @inject(ShellService) private shell: ShellService,
        @inject(WorkspaceService) private workspace: WorkspaceService,
        ) {
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
            let relativeFilePath = await this.workspace.getRelativeFilePath(arg.file);
            let workspace_uri = await this.workspace.getWorkspaceUri();
            if (relativeFilePath === undefined || workspace_uri === undefined)
                return Promise.reject("unknown file path");

            let result = this.shell.callRefactoring(relativeFilePath, arg, binaryPath)

            if(result instanceof Error) {
                this.notificationService.sendErrorNotification(result.message);
                return Promise.reject(result.message);
            }

            if (result.code === 0) {
                let edits = mapRefactorResultToWorkspaceEdit(arg, result.stdout, workspace_uri);

                await this.connection.workspace.applyEdit(edits);

                this.notificationService.sendInfoNotification(`Applied: ${arg.refactoring}`);

                return Promise.resolve();
            }
            else {
                this.notificationService.sendErrorNotification(`Refactoring failed. \nstderr: ${result.stderr}\nstdout: ${result.stdout}`);

                return Promise.reject("refactoring failed");
            }
        }
        return Promise.reject("empty argument list");
    }

}

const isValidArgs = (args: RefactorArgs) => {
	return args && args.file;
}
