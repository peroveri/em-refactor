import { Connection } from "vscode-languageserver";

export interface LSPExtensionSettings {
    isHoverEnabled: boolean;
    isGenerateTestFilesEnabled: boolean;
    refactoringBinaryPath: string;
}
export const getLSPExtensionSettings = async (connection: Connection): Promise<LSPExtensionSettings> => {
    let settings = <LSPExtensionSettings> await connection.workspace.getConfiguration({
        scopeUri: 'window',
        section: 'languageServerExample'
    });
    return settings;
}