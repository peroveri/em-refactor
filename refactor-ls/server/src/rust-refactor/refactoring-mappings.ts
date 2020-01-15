import {
    CodeAction,
    CodeActionKind,
    Command,
    Range,
    TextDocument,
    TextDocumentEdit,
    TextDocuments,
    TextEdit,
    WorkspaceFolder,
    ApplyWorkspaceEditParams,
} from 'vscode-languageserver';

import config from '../config';
import { ByteRange } from '../ls-mappings/ByteRange';

export interface RefactorArgs {
    file: string;
    version: number;
    refactoring: string;
    selection: string;
    unsafe: boolean;
}
interface Change {
    file_name: string;
    start: number;
    end: number;
    replacement: string;
}


const mapToRefactorArgs = (doc: TextDocument, range: ByteRange, refactoring: string, unsafe: boolean): RefactorArgs => ({
    file: doc.uri,
    version: doc.version,
    selection: range.toArgumentString(),
    refactoring,
    unsafe
});

const mapToCodeAction = (range: ByteRange, refactoring: string, doc: TextDocument, unsafe: boolean): CodeAction => ({
    title: `Refactor - ${refactoring}: ${range.toArgumentString()}` + (unsafe ? ' - unsafe' : ''),
    command: {
        title: 'refactor',
        command: CodeActionKind.RefactorExtract + '.function', // TODO: this should be something else
        arguments: [mapToRefactorArgs(doc, range, refactoring, unsafe)]
    },
    kind: CodeActionKind.RefactorExtract + '.function'
});

/**
 * TODO: Query the refactoring tool for possible refactorings at a given range.
 */
export function listActionsForRange(doc: TextDocument, range: Range): (Command | CodeAction)[] {

    const byteRange = ByteRange.fromRange(range, doc);
    if (!byteRange.isRange() || byteRange.isEmpty()) {
        return [];
    }

    return [
        mapToCodeAction(byteRange, 'box-field', doc, false),
        mapToCodeAction(byteRange, 'extract-block', doc, false),
        mapToCodeAction(byteRange, 'extract-method', doc, false),
        mapToCodeAction(byteRange, 'introduce-closure', doc, false),
        mapToCodeAction(byteRange, 'box-field', doc, true),
        mapToCodeAction(byteRange, 'extract-block', doc, true),
        mapToCodeAction(byteRange, 'extract-method', doc, true),
        mapToCodeAction(byteRange, 'introduce-closure', doc, true),
    ];
}

const concatUris = (uri: string, relativePath: string) =>
    uri + "/" + relativePath; // TODO: combine properly

const mapChange = (doc: TextDocument | undefined, change: Change): TextEdit => {
    if (doc === undefined) throw "document was undefined"; // doc shouldn't be undefined here
    return {
        newText: change.replacement,
        range: {
            start: doc.positionAt(change.start),
            end: doc.positionAt(change.end)
        }
    };
};

const mapDocumentChanges = (changes: Change[], workspaceUri: string, documents: TextDocuments): TextDocumentEdit[] =>
    changes.map(change => ({
        doc: documents.get(concatUris(workspaceUri, change.file_name)),
        change: change
    }))
        .filter(e => e.doc !== undefined)
        .map(edit => ({
            edits: [
                mapChange(edit.doc, edit.change)
            ],
            textDocument: {
                uri: (edit.doc ? edit.doc.uri : ''),
                version: null
            }
        } as TextDocumentEdit));

export const mapRefactorResultToWorkspaceEdit = (arg: RefactorArgs, stdout: string, workspaceUri: string, documents: TextDocuments): ApplyWorkspaceEditParams => ({
    edit: {
        documentChanges: mapDocumentChanges(JSON.parse(stdout) as [Change], workspaceUri, documents)
    },
    label: arg.refactoring
});

export const getFileRelativePath = (fileUri: string, workspace: WorkspaceFolder[] | null) => {
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

export const convertToCmd = (relativeFilePath: string, refactoring: string, selection: string, new_fn: string | null, unsafe: boolean) => {
    const refactorArgs = `--output-changes-as-json --ignore-missing-file --file=${relativeFilePath} --refactoring=${refactoring} --selection=${selection}` + (new_fn === null ? '' : ` --new_function=${new_fn}`) + (unsafe ? ' --unsafe' : '');

    // The +nightly version should match the one used in the refactoring crate
    return config.useBin ?
    `${config.refactorBinPath} ${refactorArgs}`
    : `cargo +nightly run --bin cargo-my-refactor --manifest-path=${config.refactorToolManifestPath} -- ${refactorArgs}`;
}

export const convertToCmdProvideType = (relativeFilePath: string, selection: string) => {
    const refactorArgs = `--output-changes-as-json --ignore-missing-file --file=${relativeFilePath} --provide-type --selection=${selection}`;

    // The +nightly version should match the one used in the refactoring crate
    return config.useBin ? 
    `${config.refactorBinPath} ${refactorArgs}`
    : `cargo +nightly run --bin cargo-my-refactor --manifest-path=${config.refactorToolManifestPath} -- ${refactorArgs}`;
}
