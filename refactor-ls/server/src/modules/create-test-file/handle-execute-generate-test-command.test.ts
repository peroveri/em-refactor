import * as assert from 'assert';
import { handleExecuteGenerateTestCommand } from './handle-execute-generate-test-command';
import { ApplyWorkspaceEditParams, CreateFile, ExecuteCommandParams, Position, TextDocumentEdit, TextEdit } from 'vscode-languageserver';

const expectedJson = `{
${"\t"}"file": "main.rs",
${"\t"}"args": {
${"\t"}${"\t"}"refactoring": "refactoring..",
${"\t"}${"\t"}"selection": "1:2"
${"\t"}},
${"\t"}"expected": {
${"\t"}${"\t"}"code": 0,
${"\t"}${"\t"}"stdout_file": "main_after.rs"
${"\t"}}
}`;

describe("handle-execute-generate-test-command", () => {
    describe("handleExecuteGenerateTestCommand", () => {
        it("Should map params to workspace edits", () => {
            let params: ExecuteCommandParams = {
                command: "mrefactor.generate_test_file",
                arguments: [{
                    refactoring: "refactoring..",
                    file_uri: "src/main.rs",
                    selection: "1:2",
                    should_fail: false
                }]
            };
            let expected: ApplyWorkspaceEditParams[] = [{
                edit: {
                    documentChanges: [
                        CreateFile.create("src/main.json", { overwrite: true })
                    ]
                },
                label: "refactor.generate_test_file"
            }, {
                edit: {
                    documentChanges: [
                        TextDocumentEdit.create({
                            uri: "src/main.json",
                            version: null   
                        }, [
                            TextEdit.insert(Position.create(0, 0), expectedJson)
                        ])
                    ]
                },
                label: "refactor.generate_test_file"
            }];
            let actual = handleExecuteGenerateTestCommand(params);

            assert.deepStrictEqual(actual, expected);
        })
    });
});