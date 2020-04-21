import { singleton, inject } from "tsyringe";
import * as shell from "shelljs";
import { NotificationService } from "./NotificationService";
import { RefactorArgs } from "./mappings/code-action-refactoring-mappings";

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

    callRefactoring(relativeFilePath: string, arg: RefactorArgs, binaryPath: string) {

        let cmd = convertToCmd(relativeFilePath, arg.refactoring, arg.selection, arg.unsafe, binaryPath);
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

const convertToCmd = (relativeFilePath: string, refactoring: string, selection: string, unsafe: boolean, binaryPath: string): string | Error => {
    if (!isValidBinaryPath(binaryPath)) {
        return new Error(`'${binaryPath}' is not a valid binary file`);
    }
    const refactorArgs = `refactor ${refactoring} ${relativeFilePath} ${selection} --output-replacements-as-json` + (unsafe ? ' --unsafe' : '');

    return `${binaryPath} ${refactorArgs}`;
}

const isValidBinaryPath = (binaryPath: string): boolean =>
    !!binaryPath && binaryPath.length > 0;
