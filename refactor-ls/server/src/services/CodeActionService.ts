import { singleton, inject, container } from "tsyringe";
import { TextDocuments, CodeActionParams, Command, CodeAction, TextDocument } from 'vscode-languageserver';
import { generateJsonCodeActions, listActionsForRange } from "../modules";
import { SettingsService } from "./SettingsService";

export const refactorings = [
    "box-field",
    "extract-block",
    "extract-method",
    "inline-macro",
    "introduce-closure"
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
        return listCodeActions(doc, params, settings.isGenerateTestFilesEnabled);
    };
}

// Refactorings should be shown when they are applicable at the current selection
// - From characters: e.g. left is ' ' or ';' or ...
// - From syntax: selection start is at a statement (for extract block)

const listGenerateJsonCodeActions = (doc: TextDocument, params: CodeActionParams, isGenerateTestFilesEnabled: boolean) =>
    isGenerateTestFilesEnabled ? generateJsonCodeActions(refactorings, doc, params) : [];

const listCodeActions = (doc: TextDocument, params: CodeActionParams, isGenerateTestFilesEnabled: boolean) =>
    listGenerateJsonCodeActions(doc, params, isGenerateTestFilesEnabled)
        .concat(listActionsForRange(doc, params.range, refactorings))
        .sort((a, b) => a.title.localeCompare(b.title));