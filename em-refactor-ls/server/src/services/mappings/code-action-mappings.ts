import { TextDocument, CodeActionParams, CodeActionKind } from "vscode-languageserver";
import { config, generateJsonCodeActions, listRefactorCodeActions } from ".";

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

const listGenerateJsonCodeActions = (doc: TextDocument, params: CodeActionParams, isGenerateTestFilesEnabled: boolean, isMicroRefactoringsShown: boolean) =>
    isGenerateTestFilesEnabled ? generateJsonCodeActions(listRefactorings(isMicroRefactoringsShown), doc, params) : [];

export const listCodeActions = (doc: TextDocument, params: CodeActionParams, isGenerateTestFilesEnabled: boolean, isUnsafeRefactoringShown: boolean, isMicroRefactoringsShown: boolean) =>
    listGenerateJsonCodeActions(doc, params, isGenerateTestFilesEnabled, isMicroRefactoringsShown)
        .concat(listRefactorCodeActions(doc, params.range, listRefactorings(isMicroRefactoringsShown), isUnsafeRefactoringShown))
        .sort((a, b) => a.title.localeCompare(b.title));

export const listAllCommands = () => [
        config.refactorCommand,
        config.generateTestJsonCommand,
        config.candidatesCommand,
        config.cargoCheckCommand,
    ];

export const listAllCodeActionKinds = () => [
        CodeActionKind.Refactor
    ];