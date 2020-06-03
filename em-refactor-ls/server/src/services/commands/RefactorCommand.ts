import { singleton, inject } from "tsyringe";
import { ExecuteCommandParams } from 'vscode-languageserver';
import { RefactorOutputs, mapRefactorResultToWorkspaceEdits, config } from "../mappings";
import { RefactorArgs } from '../../models';
import { NotificationService } from "../NotificationService";
import { ShellService } from "../ShellService";
import { WorkspaceService } from "../WorkspaceService";

@singleton()
export class RefactorCommand {
    constructor(
        @inject(NotificationService) private notificationService: NotificationService,
        @inject(ShellService) private shell: ShellService,
        @inject(WorkspaceService) private workspace: WorkspaceService,
    ) {
    }

    canHandle = (params: ExecuteCommandParams) => 
        params.command === config.refactorCommand;

    excuteCommand = async (params: ExecuteCommandParams) => {
        
        let arg = mapToRefactorArgs(params);
        if (arg === undefined) {
            return Promise.reject(`invalid args: ${JSON.stringify(params.arguments)}`);
        }

        let workspaceInfo = await this.workspace.getWorkspaceUri();
        let relativeFilePath = workspaceInfo?.getFileRelativePath(arg.file);
        if (workspaceInfo === undefined || relativeFilePath === undefined) {
            return Promise.reject("unknown file path");
        }

        let result = await this.shell.callRefactoring(relativeFilePath, arg)

        if (result instanceof Error) {
            this.notificationService.sendErrorNotification(result.message);
            return Promise.reject(result.message);
        }

        if (result.code === 0) {
            let outputs;
            try {
                outputs = JSON.parse(result.stdout) as RefactorOutputs;
            } catch (e) {
                console.log(e);
                throw e;
            }

            if (outputs.errors.length > 0) {
                this.notificationService.sendErrorNotification(outputs.errors[0].message);
                return Promise.reject(outputs.errors[0].message);
            }

            let edits = mapRefactorResultToWorkspaceEdits(arg, outputs, workspaceInfo.uri);

            for (const edit of edits) {

                this.notificationService.logError(JSON.stringify(edit));
                let editResponse = await this.workspace.applyEdit(edit);

                if (editResponse.applied) {
                    this.notificationService.sendInfoNotification(`Applied: ${arg.refactoring}`);
                } else {
                    this.notificationService.sendErrorNotification(`Failed to apply: ${arg.refactoring}`);
                }

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
