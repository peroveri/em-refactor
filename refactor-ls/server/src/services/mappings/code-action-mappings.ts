import { TextDocument, CodeActionParams, CodeActionKind } from "vscode-languageserver";
import { config, generateJsonCodeActions, listRefactorCodeActions } from ".";

const refactorings = [
    "box-field",
    "close-over-variables",
    "convert-closure-to-function",
    "extract-block",
    "extract-method",
    "inline-macro",
    "introduce-closure",
    "pull-up-item-declaration"
];

// Refactorings should be shown when they are applicable at the current selection
// - From characters: e.g. left is ' ' or ';' or ...
// - From syntax: selection start is at a statement (for extract block)

// Maybe use the syn library and get the tokens of a .rs file when it is opened / changed?

const listGenerateJsonCodeActions = (doc: TextDocument, params: CodeActionParams, isGenerateTestFilesEnabled: boolean) =>
    isGenerateTestFilesEnabled ? generateJsonCodeActions(refactorings, doc, params) : [];

export const listCodeActions = (doc: TextDocument, params: CodeActionParams, isGenerateTestFilesEnabled: boolean, isUnsafeRefactoringShown: boolean) =>
    listGenerateJsonCodeActions(doc, params, isGenerateTestFilesEnabled)
        .concat(listRefactorCodeActions(doc, params.range, refactorings, isUnsafeRefactoringShown))
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