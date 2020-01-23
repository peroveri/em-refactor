import { singleton } from "tsyringe";
import * as shell from "shelljs";
import { convertToCmdProvideType } from "../modules";

@singleton()
export class ShellService {
    constructor() { }

    getHoverInfo(relativeFilePath: string, selection: string, binaryPath: string) {
        let cmd = convertToCmdProvideType(relativeFilePath, selection, binaryPath);
        if (cmd instanceof Error) {
            return cmd;
        }
        /* https://github.com/shelljs/shelljs/wiki/Electron-compatibility */
        if (shell.config.execPath === null) {
            shell.config.execPath = shell.which('node').toString();
        }
        let result = shell.exec(cmd);
        if (result.code === 0) {
            let res = JSON.parse(result.stdout) as Array<{
                type: string;
            }>;
            let content = res && res.length > 0 ? res[0].type : '<empty>';
            return trimWhitespace(content);

        }
        return new Error(`command failed with code ${result.code}`);
    }
}

const trimWhitespace = (s: string) =>
    s.replace(/\n([ \t]+)/g, (match, p1: string) => {
        return '\n' + ' '.repeat((p1.length) / 8);
    });