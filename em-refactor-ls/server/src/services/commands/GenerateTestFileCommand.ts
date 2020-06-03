import { singleton, inject } from "tsyringe";
import { ExecuteCommandParams } from 'vscode-languageserver';
import { canExecuteGenerateTestCommand, handleExecuteGenerateTestCommand } from "../mappings";
import { SettingsService } from "../SettingsService";
import { WorkspaceService } from "../WorkspaceService";

@singleton()
export class GenerateTestFileCommand {
    constructor(
        @inject(SettingsService) private settings: SettingsService,
        @inject(WorkspaceService) private workspace: WorkspaceService,
    ) {
    }

    canHandle = async (params: ExecuteCommandParams) => {
        let settings = await this.settings.getSettings();

        return settings.isGenerateTestFilesEnabled && canExecuteGenerateTestCommand(params);
    }

    excuteCommand = (params: ExecuteCommandParams) => {
        const edits = handleExecuteGenerateTestCommand(params);
        return this.workspace.applyEdits(edits);
    }
}