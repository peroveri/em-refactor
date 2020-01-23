import { singleton, inject } from "tsyringe";
import { Connection, TextDocuments, TextDocumentPositionParams, Hover } from 'vscode-languageserver';
import { showTypeOrMacroExpansion } from "../modules";
import { SettingsService } from "./SettingsService";
@singleton()
export class HoverService {
    constructor(
        @inject("Connection") private connection: Connection,
        @inject("TextDocuments") private documents: TextDocuments,
        @inject(SettingsService) private settings: SettingsService) {
    }

    handleOnHover = async (params: TextDocumentPositionParams): Promise<Hover | null> => {
        let settings = await this.settings.getSettings();
        if (settings.isHoverEnabled) {
            return showTypeOrMacroExpansion(params, this.connection, this.documents, settings.refactoringBinaryPath);
        }
        return Promise.resolve(null);
    };
}
