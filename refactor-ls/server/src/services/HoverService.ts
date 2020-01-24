import { singleton, inject } from "tsyringe";
import { TextDocuments, TextDocumentPositionParams, Hover, MarkedString, Position, TextDocument } from 'vscode-languageserver';
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
        const workspaceInfo = await this.connectionService.getWorkspaceUri();
        const relativeFilePath = workspaceInfo?.getFileRelativePath(params.textDocument.uri);
        if (relativeFilePath === undefined) {
            return Promise.reject("unknown file path");
        }
        const doc = this.documents.get(params.textDocument.uri);
        if (doc === undefined) {
            return Promise.reject();
        }
        let content = this.shell.getHoverInfo(relativeFilePath, mapPositionToString(doc, params), binaryPath);

        if (content instanceof Error) {
            return Promise.reject(content.message);
        }
        return Promise.resolve(mapToHoverInfo(content, params.position));
    }
}
const mapToHoverInfo = (content: string, position: Position): Hover => ({
    contents: {
        language: 'rust',
        value: content
    } as MarkedString,
    range: {
        start: position,
        end: position
    }
});

function mapPositionToString(doc: TextDocument, params: TextDocumentPositionParams) {
    let pos = doc.offsetAt(params.position);
    return `${pos}:${pos}`;
}

