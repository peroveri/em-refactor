import { singleton, inject } from "tsyringe";
import { Connection } from 'vscode-languageserver';

@singleton()
export class SettingsService {
    constructor(@inject("Connection") private connection: Connection) { }

    getSettings() {
        return this.connection.workspace.getConfiguration({
            scopeUri: 'window',
            section: 'emRefactor'
        }).then(e => <LSPExtensionSettings>e);
    }
}

export interface LSPExtensionSettings {
    isGenerateTestFilesEnabled: boolean;
    isMicroRefactoringsShown: boolean;
    isUnsafeRefactoringShown: boolean;
    refactoringCargoTomlPath: string;
    cargoToolchain: string;
}
