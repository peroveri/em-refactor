import { singleton, inject } from "tsyringe";
import { ExecuteCommandParams } from 'vscode-languageserver';
import { handleExecuteGenerateTestCommand } from "./mappings";
import { SettingsService } from "../SettingsService";
import { WorkspaceService } from "../WorkspaceService";
import { config } from "../../models";

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