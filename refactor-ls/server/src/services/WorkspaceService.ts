import { singleton, inject } from "tsyringe";
import { Connection, ApplyWorkspaceEditParams } from "vscode-languageserver";
import { getFileRelativePath } from "./mappings/workspace-mappings";

@singleton()
export class WorkspaceService {
    constructor(
        @inject("Connection") private connection: Connection,
    ) { }

    getRelativeFilePath(uri: string) {
        return this.connection.workspace.getWorkspaceFolders()
            .then(workspaceFolders => getFileRelativePath(uri, workspaceFolders));
    }
    getWorkspaceUri() {
        return this.connection.workspace.getWorkspaceFolders()
            .then(f => f === null ? undefined : f[0].uri);
    }

    async applyEdits(params: ApplyWorkspaceEditParams[]) {
        for (const edit of params) {
            await this.applyEdit(edit);
        }
    }
    applyEdit(params: ApplyWorkspaceEditParams) {
        return this.connection.workspace.applyEdit(params);
    }
}
