import { TextDocument, CodeActionParams } from "vscode-languageserver";
import { generateJsonCodeActions, listRefactorCodeActions } from ".";
import { LSPExtensionSettings } from '../SettingsService';

const microRefactorings = [
    "close-over-variables",
    "convert-closure-to-function",
    "extract-block",
    "inline-macro",
    "introduce-closure",
    "lift-function-declaration",
    "pull-up-item-declaration"
];
const compositeRefactorings = [
    "box-field",
    "extract-method",
];

// Refactorings should be shown when they are applicable at the current selection
// - From characters: e.g. left is ' ' or ';' or ...
// - From syntax: selection start is at a statement (for extract block)

// Maybe use the syn library and get the tokens of a .rs file when it is opened / changed?

const listRefactorings = (isMicroRefactoringsShown: boolean) => {
    let refactorings = isMicroRefactoringsShown ? compositeRefactorings.concat(microRefactorings) : compositeRefactorings;
    return refactorings.sort();
};

const listGenerateJsonCodeActions = (doc: TextDocument, params: CodeActionParams, settings: LSPExtensionSettings) =>
    settings.isGenerateTestFilesEnabled ? generateJsonCodeActions(listRefactorings(settings.isMicroRefactoringsShown), doc, params) : [];

export const listCodeActions = (doc: TextDocument, params: CodeActionParams, settings: LSPExtensionSettings) =>
    listGenerateJsonCodeActions(doc, params, settings)
        .concat(listRefactorCodeActions(doc, params.range, listRefactorings(settings.isMicroRefactoringsShown), settings.isUnsafeRefactoringShown))
        .sort((a, b) => a.title.localeCompare(b.title));
