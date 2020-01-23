import { singleton, inject } from "tsyringe";
import { Connection, TextDocuments, TextDocumentPositionParams, Hover, MarkedString } from 'vscode-languageserver';
import { getFileRelativePath, convertToCmdProvideType } from "../modules";
import { SettingsService } from "./SettingsService";
import * as shell from 'shelljs';

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
            return this.showTypeOrMacroExpansion(params, settings.refactoringBinaryPath);
        }
        return Promise.resolve(null);
    };


    async showTypeOrMacroExpansion(params: TextDocumentPositionParams, binaryPath: string): Promise<Hover> {
        let workspaceFolders = await this.connection.workspace.getWorkspaceFolders();
        let relativeFilePath = getFileRelativePath(params.textDocument.uri, workspaceFolders);
        if (relativeFilePath === undefined || workspaceFolders === null)
            return Promise.reject("unknown file path");
        const doc = this.documents.get(params.textDocument.uri);
        if (doc === undefined) {
            return Promise.reject();
        }
        let pos = doc.offsetAt(params.position);
        let cmd = convertToCmdProvideType(relativeFilePath, `${pos}:${pos}`, binaryPath);
        if (cmd instanceof Error) {
            return Promise.reject(cmd.message);
        }
        /* https://github.com/shelljs/shelljs/wiki/Electron-compatibility */
        if (shell.config.execPath === null) {
            shell.config.execPath = shell.which('node').toString();
        }
        let result = shell.exec(cmd);
        if (result.code === 0) {
            let res = JSON.parse(result.stdout) as Array<{
                type: string;
            }>;
            let content = res && res.length > 0 ? res[0].type : '<empty>';
            content = content.replace(/\n([ \t]+)/g, (match, p1: string) => {
                return '\n' + ' '.repeat((p1.length) / 8);
            });
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
        return Promise.reject("refactoring failed");
    }

}
