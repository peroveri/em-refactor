import { singleton, inject } from "tsyringe";
import { ExecuteCommandParams, ApplyWorkspaceEditParams } from 'vscode-languageserver';
import { canExecuteGenerateTestCommand, handleExecuteGenerateTestCommand } from "../modules";
import { SettingsService } from "./SettingsService";
import { NotificationService } from "./NotificationService";
import { ShellService } from "./ShellService";
import { WorkspaceService } from "./WorkspaceService";
import { mapRefactorResultToWorkspaceEdit, mapOutputToCrateList, getErrors } from "./mappings/workspace-mappings"
import { RefactorArgs } from "./mappings/code-action-refactoring-mappings";

@singleton()
export class ExecuteCommandService {
    constructor(
        @inject(SettingsService) private settings: SettingsService,
        @inject(NotificationService) private notificationService: NotificationService,
        @inject(ShellService) private shell: ShellService,
        @inject(WorkspaceService) private workspace: WorkspaceService,
    ) {
    }

    handleExecuteCommand = async (params: ExecuteCommandParams): Promise<ApplyWorkspaceEditParams | void | any> => {
        try {
            let settings = await this.settings.getSettings();
            if (settings.isGenerateTestFilesEnabled && canExecuteGenerateTestCommand(params)) {
                const edits = handleExecuteGenerateTestCommand(params);
                await this.workspace.applyEdits(edits);
                return Promise.resolve();
            }
            return this.handleExecuteRefactoringCommand(params, settings.refactoringBinaryPath);
        } catch (e) {
            return Promise.reject(`Unhandled expection in handleExecuteCommand:\n${JSON.stringify(e)}`);
        }
    };


    async handleExecuteRefactoringCommand(params: ExecuteCommandParams, binaryPath: string): Promise<ApplyWorkspaceEditParams | void> {

        let arg = mapToRefactorArgs(params);
        if (arg === undefined) {
            return Promise.reject(`invalid args: ${JSON.stringify(params.arguments)}`);
        }

        let workspaceInfo = await this.workspace.getWorkspaceUri();
        let relativeFilePath = workspaceInfo?.getFileRelativePath(arg.file);
        if (workspaceInfo === undefined || relativeFilePath === undefined) {
            return Promise.reject("unknown file path");
        }

        let result = this.shell.callRefactoring(relativeFilePath, arg, binaryPath)

        if (result instanceof Error) {
            this.notificationService.sendErrorNotification(result.message);
            return Promise.reject(result.message);
        }

        if (result.code === 0) {
            let outputs;
            try {
                outputs = mapOutputToCrateList(result.stdout);
            } catch(e) {
                console.log(e);
                throw e;
            }

            let errors = getErrors(outputs);
            if(errors.length > 0) {
                this.notificationService.sendErrorNotification(errors[0].message);
                return Promise.reject(errors[0].message);
            }
            
            let edit = mapRefactorResultToWorkspaceEdit(arg, outputs, workspaceInfo.uri);

            this.notificationService.logError(JSON.stringify(edit));
            let editResponse = await this.workspace.applyEdit(edit);
            
            if(editResponse.applied) {
                this.notificationService.sendInfoNotification(`Applied: ${arg.refactoring}`);
            } else {
                this.notificationService.sendErrorNotification(`Failed to apply: ${arg.refactoring}`);
            }


            return Promise.resolve();
        } else {
            this.notificationService.sendErrorNotification(`Refactoring failed. \nstderr: ${result.stderr}\nstdout: ${result.stdout}`);

            return Promise.reject("refactoring failed");
        }
    }

}

const mapToRefactorArgs = (params: ExecuteCommandParams): RefactorArgs | undefined => {
    if (params && params.arguments && params.arguments[0]) {
        let arg = params.arguments[0] as RefactorArgs;
        if (!arg || !arg.file) {
            return undefined;
        }
        return arg;
    }

    return undefined;
}
