import { singleton, inject } from "tsyringe";
import { Connection } from 'vscode-languageserver';

@singleton()
export class SettingsService {
    constructor(@inject("Connection") private connection: Connection) { }

    getSettings() {
        return this.connection.workspace.getConfiguration({
            scopeUri: 'window',
            section: 'languageServerExample'
        }).then(e => <LSPExtensionSettings>e);
    }
}

interface LSPExtensionSettings {
    isGenerateTestFilesEnabled: boolean;
    isUnsafeRefactoringShown: boolean;
    refactoringBinaryPath: string;
    cargoToolchain: string;
}
