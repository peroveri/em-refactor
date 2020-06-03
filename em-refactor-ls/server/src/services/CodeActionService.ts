import { singleton, inject } from "tsyringe";
import { TextDocuments, CodeActionParams, Command, CodeAction } from 'vscode-languageserver';
import { SettingsService } from "./SettingsService";
import { listGenerateTestFileCodeActions, listRefactorCodeActions } from "./code-actions";

@singleton()
export class CodeActionService {
    constructor(
        @inject("TextDocuments") private documents: TextDocuments,
        @inject(SettingsService) private settings: SettingsService) {
    }

    /**
     * Handles the textDocument/codeAction request
     */
    handleCodeAction = async (params: CodeActionParams): Promise<(Command | CodeAction)[]> => {
        let settings = await this.settings.getSettings();
        const doc = this.documents.get(params.textDocument.uri);
        if (doc === undefined) {
            return Promise.resolve([]);
        }
        return listGenerateTestFileCodeActions(doc, params, settings)
            .concat(listRefactorCodeActions(doc, params.range, settings))
            .sort((a, b) => a.title.localeCompare(b.title));;
    };
}
