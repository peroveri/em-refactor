/*
 * Test execution of the extract-block refactoring 
*/

import * as vscode from 'vscode';
import * as assert from 'assert';
import { activate, doc, getDocUri, setTestContent } from '../helper';

const mainrs = `fn main() {
    let s = "Hello, world!";
    println!("{}", s);
}
`;
const expectedMain = `fn main() {
    let s = 
{
let s = "Hello, world!";
s};
    println!("{}", s);
}
`;

const executeExtractBlockCommand = (file: string, start: number, end: number) => 
    vscode.commands.executeCommand(
        'mrefactor.refactor',
        {
            file,
            version: null,
            selection: `${start}:${end}`,
            refactoring: "extract-block",
            unsafe: false
        }
    );

const assertCommandExists = async (name: string) => {
    let actual = (await vscode.commands.getCommands(true)).indexOf(name);
    assert.notStrictEqual(actual, -1);
}

suite('executeCommand', () => {
    
    setup(async () => {
        await activate(getDocUri("src/main.rs"), getDocUri(""));
    });

    suite("extract-block", () => {

        test('Should have refactor command', async () => assertCommandExists('mrefactor.refactor'));

        test('Should extract block and set content', async () => {
            assert.equal(await setTestContent(mainrs), true);

            let commandResponse = await executeExtractBlockCommand(getDocUri("src/main.rs").toString(), 16, 40);
            
            let text = doc.getText();
            assert.equal(commandResponse, null);
            assert.strictEqual(text, expectedMain);
        });
    });
});
