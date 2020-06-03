import { singleton, inject } from "tsyringe";
import { ExecuteCommandParams } from 'vscode-languageserver';
import { handleExecuteGenerateTestCommand, config } from "../mappings";
import { SettingsService, WorkspaceService } from "../";

@singleton()
export class GenerateTestFileCommand {
    constructor(
        @inject(SettingsService) private settings: SettingsService,
        @inject(WorkspaceService) private workspace: WorkspaceService,
    ) {
    }

    canHandle = async (params: ExecuteCommandParams) => {
        let settings = await this.settings.getSettings();

        return params.command === config.generateTestJsonCommand &&
               settings.isGenerateTestFilesEnabled;
    }

    excuteCommand = (params: ExecuteCommandParams) => {
        const edits = handleExecuteGenerateTestCommand(params);
        return this.workspace.applyEdits(edits);
    }
}