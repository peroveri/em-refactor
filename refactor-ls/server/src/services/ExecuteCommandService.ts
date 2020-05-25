import { singleton, inject } from "tsyringe";
import { ExecuteCommandParams, ApplyWorkspaceEditParams, TextEdit, CreateFile, TextDocumentEdit, Position } from 'vscode-languageserver';
import { canExecuteGenerateTestCommand, config, handleExecuteGenerateTestCommand, mapRefactorResultToWorkspaceEdits, RefactorArgs, RefactorOutputs } from "./mappings";
import { SettingsService } from "./SettingsService";
import { NotificationService } from "./NotificationService";
import { ShellService } from "./ShellService";
import { WorkspaceService } from "./WorkspaceService";

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
            } else if(await this.handleCustom(params)) {

                return Promise.resolve();
            }
            return this.handleExecuteRefactoringCommand(params);
        } catch (e) {
            return Promise.reject(`Unhandled expection in handleExecuteCommand:\n${JSON.stringify(e)}`);
        }
    };

    handleCustom = async (params: ExecuteCommandParams) => {
        if(params.command === config.cargoCheckCommand) {
            await this.shell.runCargoCheck();
            return true;
        } else if (params.command === config.candidatesCommand && params.arguments && params.arguments[0]) {
            let res = await this.shell.queryCandidates(params.arguments[0]);
            if(res instanceof Error) {
                this.notificationService.sendErrorNotification(res.message);
            } else {
                let workspaceInfo = await this.workspace.getWorkspaceUri();
                let newFileUri = workspaceInfo?.join(`${params.arguments[0]}-candidates.json`);
                if(newFileUri !== undefined) {
                    let edits = newFile(newFileUri, res);
                    await this.workspace.applyEdits(edits);
                }
            }
            return true;
        }
        return false;
    }

    async handleExecuteRefactoringCommand(params: ExecuteCommandParams): Promise<ApplyWorkspaceEditParams | void> {

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
            } catch(e) {
                console.log(e);
                throw e;
            }

            if(outputs.errors.length > 0) {
                this.notificationService.sendErrorNotification(outputs.errors[0].message);
                return Promise.reject(outputs.errors[0].message);
            }
            
            let edits = mapRefactorResultToWorkspaceEdits(arg, outputs, workspaceInfo.uri);

            for(const edit of edits) {

                this.notificationService.logError(JSON.stringify(edit));
                let editResponse = await this.workspace.applyEdit(edit);
                
                if(editResponse.applied) {
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

const newFile = (name: string, content: string) => 
    [{
        edit: {
            documentChanges: [
                CreateFile.create(name, { overwrite: true }),
            ],
            label: config.candidatesCommand
        }
    }, {
        edit: {
            documentChanges: [
                TextDocumentEdit.create({
                    uri: name,
                    version: null
                }, [
                    TextEdit.insert(Position.create(0, 0), content)
                ])
            ],
            label: config.candidatesCommand
        },
    }];

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
