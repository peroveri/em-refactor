import {
    CodeAction,
    CodeActionParams,
    Command,
    TextDocument,
    ExecuteCommandParams,
    ApplyWorkspaceEditParams,
    TextDocumentIdentifier,
    CodeActionKind,
    Position,
    CreateFile,
    TextEdit,
    TextDocumentEdit,
} from 'vscode-languageserver';

import {
    ByteRange
} from './refactoring-mappings';

const refactorings = [
    'box-field',
    'extract-block'
];

interface GenerateTestFileArgs {
    refactoring: string;
    file_uri: string;
    selection: string;
    should_fail: boolean;
}

const mapToCodeAction = (params: CodeActionParams, refactoring: string, should_fail: boolean, selection: string): CodeAction => ({
    title: `Generate ${getDocName(params.textDocument)}.json for ${refactoring} ${should_fail ? ' (failing)' : ''}`,
    command: {
        title: 'generate',
        command: `${CodeActionKind.Refactor}.generate_test_file`,
        arguments: [{
            file_uri: params.textDocument.uri,
            refactoring,
            selection,
            should_fail
        } as GenerateTestFileArgs]
    },
    kind: `${CodeActionKind.Refactor}.generate_test_file`
});

const getDocName = (doc: TextDocumentIdentifier): string =>
    doc.uri.substring(doc.uri.lastIndexOf("/") + 1, doc.uri.lastIndexOf("."));

function generate_json_actions(document: TextDocument, params: CodeActionParams): (Command | CodeAction)[] {
    let byteRange = ByteRange.fromRange(params.range, document);
    if (!byteRange.isRange()) {
        return [];
    }

    return refactorings
        .map(refactoring => mapToCodeAction(params, refactoring, true, byteRange.toArgumentString()))
        .concat(refactorings
            .map(refactoring => mapToCodeAction(params, refactoring, false, byteRange.toArgumentString())));
}

const canExecuteGenerateTestCommand = (params: ExecuteCommandParams) =>
    `${CodeActionKind.Refactor}.generate_test_file` === params.command;

const getTestName = (s: string): string =>
    s.substring(s.lastIndexOf("/") + 1, s.lastIndexOf(".rs"));

const getJsonContent = (args: GenerateTestFileArgs): string =>
    JSON.stringify({
        "file": `${getTestName(args.file_uri)}.rs`,
        "args": {
            "refactoring": args.refactoring,
            "selection": args.selection
        },
        "expected": args.should_fail ? {
            "code": 2,
            "stderr": "todo"
        } : {
                "code": 0,
                "stdout_file": `${getTestName(args.file_uri)}_after.rs`
            }
    }, undefined, '\t');

const getJsonFileName = (uri: string): string | null => {
    if (!uri.endsWith(".rs")) {
        return null;
    }

    return uri.substring(0, uri.lastIndexOf(".rs")) + ".json";
}

async function handleExecuteGenerateTestCommand(params: ExecuteCommandParams): Promise<ApplyWorkspaceEditParams[]> {
    let args = params.arguments as GenerateTestFileArgs[];
    if (!args || args.length !== 1) {
        return Promise.reject();
    }
    let jsonFileName = getJsonFileName(args[0].file_uri);
    if (jsonFileName === null) {
        return Promise.reject();
    }
    let edits: ApplyWorkspaceEditParams[] = [{
        edit: {
            documentChanges: [
                CreateFile.create(jsonFileName, { overwrite: true }),
            ]
        },
        label: `${CodeActionKind.Refactor}.generate_test_file`
    }, {
        edit: {
            documentChanges: [
                TextDocumentEdit.create({
                    uri: jsonFileName,
                    version: null
                }, [
                    TextEdit.insert(Position.create(0, 0), getJsonContent(args[0]))
                ])
            ]
        },
        label: `${CodeActionKind.Refactor}.generate_test_file`
    }];
    return Promise.resolve(edits);
}

export { canExecuteGenerateTestCommand, generate_json_actions, handleExecuteGenerateTestCommand };