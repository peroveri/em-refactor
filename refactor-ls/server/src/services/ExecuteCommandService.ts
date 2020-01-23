import { singleton, inject } from "tsyringe";
import { Connection, ExecuteCommandParams, ApplyWorkspaceEditParams } from 'vscode-languageserver';
import { canExecuteGenerateTestCommand, handleExecuteGenerateTestCommand, handleExecuteRefactoringCommand } from "../modules";
import { SettingsService } from "./SettingsService";

@singleton()
export class ExecuteCommandService {
    constructor(
        @inject("Connection") private connection: Connection,
        @inject(SettingsService) private settings: SettingsService) {
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
        return handleExecuteRefactoringCommand(params, this.connection, settings.refactoringBinaryPath);
    };
}
