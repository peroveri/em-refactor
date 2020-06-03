import { TextDocument, CodeActionParams } from "vscode-languageserver";
import { LSPExtensionSettings, listRefactorings } from "../../models";
import { generateJsonCodeActions } from "../mappings";

const listGenerateJsonCodeActions = (doc: TextDocument, params: CodeActionParams, settings: LSPExtensionSettings) =>
settings.isGenerateTestFilesEnabled ? generateJsonCodeActions(listRefactorings(settings.isMicroRefactoringsShown), doc, params) : [];

export const listCodeActions = (doc: TextDocument, params: CodeActionParams, settings: LSPExtensionSettings) =>
    listGenerateJsonCodeActions(doc, params, settings);
    