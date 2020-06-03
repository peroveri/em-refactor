import { TextDocument, CodeActionParams } from "vscode-languageserver";
import { LSPExtensionSettings, listRefactorings } from "../../models";
import { generateJsonCodeActions } from "../mappings";

export const listGenerateTestFileCodeActions = (doc: TextDocument, params: CodeActionParams, settings: LSPExtensionSettings) =>
    settings.isGenerateTestFilesEnabled ? generateJsonCodeActions(listRefactorings(settings.isMicroRefactoringsShown), doc, params) : [];
    