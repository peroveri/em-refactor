import { singleton, inject } from "tsyringe";
import * as shell from "shelljs";
import { RefactorArgs } from "../modules";
import { NotificationService } from "./NotificationService";

@singleton()
export class ShellService {
    private isExecuting: boolean = false;
    constructor(
        @inject(NotificationService) private notifications: NotificationService) { }

    private shellExec(cmd: string) {
        if(this.isExecuting) {
            return new Error("Shell already executing");
        }
        this.isExecuting = true;
        this.notifications.logInfo(`executing cmd:\n${cmd}`);
        let result = shell.exec(cmd);
        this.isExecuting = false;
        return result;
    }

    getHoverInfo(relativeFilePath: string, selection: string, binaryPath: string) {
        let cmd = convertToCmdProvideType(relativeFilePath, selection, binaryPath);
        if (cmd instanceof Error) {
            return cmd;
        }
        /* https://github.com/shelljs/shelljs/wiki/Electron-compatibility */
        if (shell.config.execPath === null) {
            shell.config.execPath = shell.which('node').toString();
        }
        let result = this.shellExec(cmd);
        if(result instanceof Error) {
            return result;
        }
        if (result.code === 0) {
            let res = JSON.parse(result.stdout) as Array<{
                type: string;
            }>;
            let content = res && res.length > 0 ? res[0].type : '<empty>';
            return trimWhitespace(content);

        }
        return new Error(`command failed with code ${result.code}`);
    }

    callRefactoring(relativeFilePath: string, arg: RefactorArgs, binaryPath: string) {

        let cmd = convertToCmd(relativeFilePath, arg.refactoring, arg.selection, arg.refactoring === 'extract-method' ? 'foo' : null, arg.unsafe, binaryPath);
        if (cmd instanceof Error) {
            return new Error(cmd.message);
        }
        /* https://github.com/shelljs/shelljs/wiki/Electron-compatibility */
        if (shell.config.execPath === null) {
            shell.config.execPath = shell.which('node').toString();
        }

        const result = this.shellExec(cmd);
        if(result instanceof Error) {
            return result;
        }
        if (result.code === 0) {
            this.notifications.logInfo(JSON.stringify(result));
        } else {
            this.notifications.logError(`Got error code: ${result.code}`);
            this.notifications.logError(result.stdout);
            this.notifications.logError(result.stderr);
        }
        return result;
    }
}

const trimWhitespace = (s: string) =>
    s.replace(/\n([ \t]+)/g, (match, p1: string) => {
        return '\n' + ' '.repeat((p1.length) / 8);
    });

const convertToCmd = (relativeFilePath: string, refactoring: string, selection: string, new_fn: string | null, unsafe: boolean, binaryPath: string): string | Error => {
    if (!isValidBinaryPath(binaryPath)) {
        return new Error(`'${binaryPath}' is not a valid binary file`);
    }
    const refactorArgs = `--output-replacements-as-json --ignore-missing-file --file=${relativeFilePath} --refactoring=${refactoring} --selection=${selection}` + (new_fn === null ? '' : ` --new_function=${new_fn}`) + (unsafe ? ' --unsafe' : '');

    return `${binaryPath} ${refactorArgs}`;
}

const convertToCmdProvideType = (relativeFilePath: string, selection: string, binaryPath: string): string | Error => {
    if (!isValidBinaryPath(binaryPath)) {
        return new Error(`'${binaryPath}' is not a valid binary file`);
    }
    const refactorArgs = `--output-changes-as-json --ignore-missing-file --file=${relativeFilePath} --provide-type --selection=${selection}`;

    return `${binaryPath} ${refactorArgs}`;
}

const isValidBinaryPath = (binaryPath: string): boolean =>
    !!binaryPath && binaryPath.length > 0;
