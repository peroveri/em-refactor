import { singleton, inject } from "tsyringe";
import { Connection, ApplyWorkspaceEditParams } from "vscode-languageserver";
import { WorkspaceFolderInfo } from "./mappings";

@singleton()
export class WorkspaceService {
    constructor(
        @inject("Connection") private connection: Connection,
    ) { }

    getWorkspaceUri() {
        return this.connection.workspace.getWorkspaceFolders()
            .then(WorkspaceFolderInfo.map);
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
