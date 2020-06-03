import * as path from "path";
import { WorkspaceFolder } from "vscode-languageserver";

export class WorkspaceFolderInfo {
    constructor(public uri: string) { }

    getFileRelativePath(fileUri: string) {
        return path.relative(this.uri, fileUri);
    }
    join(otherPath: string) {
        return path.join(this.uri, otherPath);
    }

    static map(folders: WorkspaceFolder[] | null) {
        if (folders === null || folders.length <= 0 || !folders[0].uri) {
            return undefined;
        }
        return new WorkspaceFolderInfo(folders[0].uri);
    }
}
