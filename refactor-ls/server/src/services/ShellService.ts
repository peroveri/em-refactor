import { singleton, inject } from "tsyringe";
import * as shell from "shelljs";
import { convertToCmdProvideType, RefactorArgs, convertToCmd } from "../modules";
import { NotificationService } from "./NotificationService";

@singleton()
export class ShellService {
    constructor(
        @inject(NotificationService) private notifications: NotificationService) { }

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

    callRefactoring(relativeFilePath: string, arg: RefactorArgs, binaryPath: string) {

        let cmd = convertToCmd(relativeFilePath, arg.refactoring, arg.selection, arg.refactoring === 'extract-method' ? 'foo' : null, arg.unsafe, binaryPath);
        if (cmd instanceof Error) {
            return new Error(cmd.message);
        }
        /* https://github.com/shelljs/shelljs/wiki/Electron-compatibility */
        if (shell.config.execPath === null) {
            shell.config.execPath = shell.which('node').toString();
        }

        this.notifications.logInfo(`executing cmd:\n${cmd}`);
        let result = shell.exec(cmd);

        if(result.code === 0) {
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