/*
 * Test execution of the extract-block refactoring 
*/

import * as vscode from 'vscode';
import * as assert from 'assert';
import { activate, getDocUri, getFileContent, setTestContent } from '../helper';

const mainrs = `fn main() {
    let s = "Hello, world!";
    println!("{}", s);
}
`;
const expectedTestMain = `{
    'a': 1
}`;

const executeGenerateTestFileCommand = (file: string, start: number, end: number) => 
    vscode.commands.executeCommand(
        'mrefactor.generate_test_file',
        {
            file_uri: file,
            selection: `${start}:${end}`,
            refactoring: "extract-block",
            should_fail: false
        }
    );

const assertCommandExists = async (name: string) => {
    let actual = (await vscode.commands.getCommands(true)).indexOf(name);
    assert.notStrictEqual(actual, -1);
}

suite('generate test file', () => {
    
    setup(async () => {
        await activate(getDocUri("src/main.rs"), getDocUri(""));
    });

    suite("extract-block", () => {

        test('Should have generate test command', async () => assertCommandExists('mrefactor.generate_test_file'));

        test('Should generate test file', async () => {
            assert.equal(await setTestContent(mainrs), true);

            let commandResponse = await executeGenerateTestFileCommand(getDocUri("src/main.rs").toString(), 16, 40);
            
            // await new Promise(resolve => setTimeout(resolve, 2000));

            // const text = await getFileContent(getDocUri("src/main.json").toString());
            // assert.equal(commandResponse, null);
            // assert.strictEqual(text, expectedTestMain);
        });
    });
});
