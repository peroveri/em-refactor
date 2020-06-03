import { singleton, inject } from "tsyringe";
import { Connection } from 'vscode-languageserver';
import { LSPExtensionSettings } from "../models";

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
