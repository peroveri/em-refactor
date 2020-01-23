import { singleton, inject } from "tsyringe";
import { Connection } from "vscode-languageserver";
import { getFileRelativePath } from "../modules";

@singleton()
export class ConnectionService {
    constructor(
        @inject("Connection") private connection: Connection,
    ) { }

    async getRelativeFilePath(uri: string) {
        let workspaceFolders = await this.connection.workspace.getWorkspaceFolders();
        return getFileRelativePath(uri, workspaceFolders);
    }
}