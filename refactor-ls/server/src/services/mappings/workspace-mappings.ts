import { Range, ApplyWorkspaceEditParams, TextDocumentEdit, TextEdit, WorkspaceFolder } from "vscode-languageserver";
import { RefactorArgs } from "./code-action-refactoring-mappings";
import * as path from "path";
interface CrateOutputs {
    candidates: any[],
    refactorings: CrateOutput[]
}
interface CrateOutput {
    crate_name: string;
    // root_path: string;
    is_test: boolean;
    replacements: Change[];
    errors: RefactorError[];
}

interface Change {
    byte_end: number;
    byte_start: number;
    char_end: number;
    char_start: number;
    file_name: string;
    line_end: number;
    line_start: number;
    replacement: string;
}
interface RefactorError {
    is_error: boolean;
    message: string
}

const changeEquals = (c1: Change, c2: Change) =>
    c1.char_end === c2.char_end &&
    c1.char_start === c2.char_start &&
    c1.file_name === c2.file_name &&
    c1.line_end === c2.line_end &&
    c1.line_start === c2.line_start &&
    c1.replacement === c2.replacement;

const mapRange = (change: Change): Range =>
    Range.create(change.line_start, change.char_start, change.line_end, change.char_end);

export const mapOutputToCrateList = (stdout: string) =>
    JSON.parse(stdout) as CrateOutputs;

export const mapToUnionOfChanges = (output: CrateOutput[]) => {
    const allChanges = output.map(e => e.replacements).reduce((acc, x) => acc.concat(x), []);

    for (let i = 0; i < allChanges.length; i++) {
        for (let j = i + 1; j < allChanges.length; j++) {
            if (changeEquals(allChanges[i], allChanges[j])) {
                allChanges.splice(j--, 1);
            }
        }
    }
    return allChanges;
}

export const mapRefactorResultToWorkspaceEdit = (arg: RefactorArgs, outputs: CrateOutput[], workspaceUri: string): ApplyWorkspaceEditParams => {
    let changes = mapToUnionOfChanges(outputs);

    let documentChanges: TextDocumentEdit[] = [];

    for (const change of changes) {
        let uri = path.join(workspaceUri, change.file_name);
        let documentChange = documentChanges.find(e => e.textDocument.uri === uri);
        if (documentChange === undefined) {
            documentChange = TextDocumentEdit.create({
                uri,
                version: null
            }, []);
            documentChanges.push(documentChange);
        }
        documentChange.edits.push(TextEdit.replace(mapRange(change), change.replacement));
    }
    return {
        edit: {
            documentChanges
        },
        label: arg.refactoring
    } as ApplyWorkspaceEditParams;
}

export const getErrors = (outputs: CrateOutput[]) =>
    outputs.map(e => e.errors).reduce((acc, x) => acc.concat(x), []).filter(e => e.is_error);

export class WorkspaceFolderInfo {
    constructor(public uri: string) { }

    getFileRelativePath(fileUri: string) {
        return path.relative(this.uri, fileUri);
    }

    static map(folders: WorkspaceFolder[] | null) {
        if (folders === null || folders.length <= 0 || !folders[0].uri) {
            return undefined;
        }
        return new WorkspaceFolderInfo(folders[0].uri);
    }
}
