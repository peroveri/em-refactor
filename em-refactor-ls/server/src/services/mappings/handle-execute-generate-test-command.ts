import { ExecuteCommandParams, ApplyWorkspaceEditParams, CodeActionKind, Position, CreateFile, TextEdit, TextDocumentEdit } from 'vscode-languageserver';
import { GenerateTestFileArgs } from '../../models';

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

const getTestName = (s: string): string =>
    s.substring(s.lastIndexOf("/") + 1, s.lastIndexOf(".rs"));



export const handleExecuteGenerateTestCommand = (params: ExecuteCommandParams): ApplyWorkspaceEditParams[] => {
    let args = params.arguments as GenerateTestFileArgs[];
    if (!args || args.length !== 1) {
        return [];
    }
    let jsonFileName = getJsonFileName(args[0].file_uri);
    if (jsonFileName === null) {
        return [];
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
    return edits;
}
