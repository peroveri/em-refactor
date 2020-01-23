import { singleton, inject } from "tsyringe";
import { Connection } from 'vscode-languageserver';
import { getLSPExtensionSettings } from "../modules";

@singleton()
export class SettingsService {
    constructor(@inject("Connection") private connection: Connection) { }

    getSettings() {
        return getLSPExtensionSettings(this.connection);
    }
}
