import { singleton, inject } from "tsyringe";
import { TextDocuments, CodeActionParams, Command, CodeAction } from 'vscode-languageserver';
import { SettingsService } from "./SettingsService";
import { listCodeActions } from "./mappings/code-action-mappings";

@singleton()
export class CodeActionService {
    constructor(
        @inject("TextDocuments") private documents: TextDocuments,
        @inject(SettingsService) private settings: SettingsService) {
    }

    handleCodeAction = async (params: CodeActionParams): Promise<(Command | CodeAction)[]> => {
        let settings = await this.settings.getSettings();
        const doc = this.documents.get(params.textDocument.uri);
        if (doc === undefined) {
            return Promise.resolve([]);
        }
        return listCodeActions(doc, params, settings.isGenerateTestFilesEnabled);
    };
}
