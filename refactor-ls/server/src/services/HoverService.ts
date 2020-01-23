import { singleton, inject } from "tsyringe";
import { TextDocuments, TextDocumentPositionParams, Hover, MarkedString } from 'vscode-languageserver';
import { SettingsService } from "./SettingsService";
import { ShellService } from './ShellService';
import { WorkspaceService } from './WorkspaceService';

@singleton()
export class HoverService {
    constructor(
        @inject("TextDocuments") private documents: TextDocuments,
        @inject(SettingsService) private settings: SettingsService,
        @inject(ShellService) private shell: ShellService,
        @inject(WorkspaceService) private connectionService: WorkspaceService
    ) {
    }

    handleOnHover = async (params: TextDocumentPositionParams): Promise<Hover | null> => {
        let settings = await this.settings.getSettings();
        if (settings.isHoverEnabled) {
            return this.showTypeOrMacroExpansion(params, settings.refactoringBinaryPath);
        }
        return Promise.resolve(null);
    };

    async showTypeOrMacroExpansion(params: TextDocumentPositionParams, binaryPath: string): Promise<Hover> {
        const relativeFilePath = await this.connectionService.getRelativeFilePath(params.textDocument.uri);
        if (relativeFilePath === undefined)
            return Promise.reject("unknown file path");
        const doc = this.documents.get(params.textDocument.uri);
        if (doc === undefined) {
            return Promise.reject();
        }
        let pos = doc.offsetAt(params.position);

        let content = this.shell.getHoverInfo(relativeFilePath, `${pos}:${pos}`, binaryPath);

        if(content instanceof Error) {
            return Promise.reject(content.message);
        } 
        return Promise.resolve({
            contents: {
                language: 'rust',
                value: content
            } as MarkedString,
            range: {
                start: params.position,
                end: params.position
            }
        } as Hover);
    }
}
