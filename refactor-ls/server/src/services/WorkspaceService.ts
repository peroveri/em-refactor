import { singleton, inject } from "tsyringe";
import { Connection, WorkspaceFolder, ApplyWorkspaceEditParams } from "vscode-languageserver";

@singleton()
export class WorkspaceService {
    constructor(
        @inject("Connection") private connection: Connection,
    ) { }

    async getRelativeFilePath(uri: string) {
        let workspaceFolders = await this.connection.workspace.getWorkspaceFolders();
        return getFileRelativePath(uri, workspaceFolders);
    }
    async getWorkspaceUri() {
        let f = await this.connection.workspace.getWorkspaceFolders();
        return f === null ? undefined : f[0].uri;
    }

    async applyEdits(params: ApplyWorkspaceEditParams[]) {
        for (const edit of params) {
            await this.applyEdit(edit);
        }
    }
    async applyEdit(params: ApplyWorkspaceEditParams) {
        await this.connection.workspace.applyEdit(params);
    }
}


const getFileRelativePath = (fileUri: string, workspace: WorkspaceFolder[] | null) => {
    if (workspace === null || workspace.length === 0) return undefined;
    let workspaceUri = workspace[0].uri;
    return getRelativePath(workspaceUri, fileUri);
}

const getRelativePath = (workspaceUri: string, fileUri: string) => {
    if (fileUri.startsWith(workspaceUri)) {
        let sub = fileUri.substring(workspaceUri.length);
        if (sub.startsWith("/")) sub = sub.substring(1);
        return sub;
    }
    return undefined;
}
