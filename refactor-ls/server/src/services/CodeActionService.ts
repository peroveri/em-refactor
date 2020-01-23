import { singleton, inject } from "tsyringe";
import { TextDocuments, CodeActionParams, Command, CodeAction } from 'vscode-languageserver';
import { generateJsonCodeActions, listActionsForRange } from "../modules";
import { SettingsService } from "./SettingsService";

export const refactorings = [
    "extract-block",
    "box-field"
];

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
        let result: (Command | CodeAction)[] = [];
        if (settings.isGenerateTestFilesEnabled) {
            result = result.concat(generateJsonCodeActions(refactorings, doc, params));
        }
        result = result.concat(listActionsForRange(doc, params.range));
        return Promise.resolve(result);
    };
}
