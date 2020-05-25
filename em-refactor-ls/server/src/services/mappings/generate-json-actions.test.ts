import * as assert from 'assert';
import { mapToCodeAction } from '.';
import { Position, CodeAction, CodeActionKind, CodeActionParams } from 'vscode-languageserver';

describe("generate-json-actions", () => {
    describe("mapToCodeAction", () => {
        it("Should map params to code action", () => {
            let params: CodeActionParams = {
                textDocument: {
                    uri: "some_dir/some_file.rs"
                },
                range: {// not used
                    start: Position.create(1,1),
                    end: Position.create(1,1),
                },
                context: { // not used
                    diagnostics: []
                }
            };
            let [refactoring_name, should_fail, selection] = ["a_refactoring_name", false, "10:20"];
            let expected: CodeAction = {
                title: "Generate some_file.json for a_refactoring_name ",
                command: {
                    command: "mrefactor.generate_test_file",
                    title: "generate",
                    arguments: [{
                        "file_uri": "some_dir/some_file.rs",
                        "refactoring": "a_refactoring_name",
                        "selection": "10:20",
                        "should_fail": false
                    }]
                },
                kind: CodeActionKind.Refactor
            };
            let actual = mapToCodeAction(params, refactoring_name, should_fail, selection);

            assert.deepStrictEqual(actual, expected);
        })
    });
});