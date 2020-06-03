import { singleton, inject } from "tsyringe";
import * as shell from "shelljs";
import { NotificationService } from "./NotificationService";
import { SettingsService } from "./SettingsService";
import { RefactorArgs, LSPExtensionSettings } from "../models";

@singleton()
export class ShellService {
    private isExecuting: boolean = false;
    constructor(
        @inject(SettingsService) private settings: SettingsService,
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

    async callRefactoring(relativeFilePath: string, arg: RefactorArgs) {
        let settings = await this.settings.getSettings();
        let cmd = convertToCmd(relativeFilePath, arg, settings);
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
            this.notifications.logInfo(`\nstdout: ${result.stdout}`);
            this.notifications.logInfo(`\nstderr: ${result.stderr}`);
        } else {
            this.notifications.logError(`Got error code: ${result.code}`);
            this.notifications.logError(result.stdout);
            this.notifications.logError(result.stderr);
        }
        return result;
    }

    async runCargoCheck() {
        /* https://github.com/shelljs/shelljs/wiki/Electron-compatibility */
        if (shell.config.execPath === null) {
            shell.config.execPath = shell.which('node').toString();
        }
        let settings = await this.settings.getSettings();
        let res = this.shellExec(`cargo ${settings.cargoToolchain} check --target-dir=./target/refactorings --all-targets`);
        if(res instanceof Error) {
            this.notifications.sendErrorNotification(res.message);
        } else {
            this.notifications.sendInfoNotification(`cargo check returned with status: ${res.code}`);
        }
    }
    async queryCandidates(refactoring: string) {
        /* https://github.com/shelljs/shelljs/wiki/Electron-compatibility */
        if (shell.config.execPath === null) {
            shell.config.execPath = shell.which('node').toString();
        }
        let settings = await this.settings.getSettings();
        let cmd = convertToCandidateCmd(refactoring, settings);
        if (cmd instanceof Error) {
            return new Error(cmd.message);
        }
        let res = this.shellExec(cmd);
        if(res instanceof Error) {
            return res;
        }
        if(res.code !== 0) {
            return new Error(`candidates failed with code: ${res.code}\nstderr:${res.stderr}`);
        }
        return res.toString();
    }
}

const convertToCmd = (relativeFilePath: string, arg: RefactorArgs, settings: LSPExtensionSettings): string | Error => {
    if (!isValidBinaryPath(settings.refactoringCargoTomlPath)) {
        return new Error(`'${settings.refactoringCargoTomlPath}' is not a path`);
    }
    const refactorArgs = `--target-dir=./target/refactorings refactor ${arg.refactoring} ${relativeFilePath} ${arg.selection}` + (arg.unsafe ? ' --unsafe' : '');

    return `cargo ${settings.cargoToolchain} run --bin cargo-em-refactor --manifest-path=${settings.refactoringCargoTomlPath} -- ${refactorArgs}`;
}
const convertToCandidateCmd = (refactoring: string, settings: LSPExtensionSettings): string | Error => {
    if (!isValidBinaryPath(settings.refactoringCargoTomlPath)) {
        return new Error(`'${settings.refactoringCargoTomlPath}' is not a path`);
    }
    const refactorArgs = `--target-dir=./target/refactorings candidates ${refactoring}`;

    return `cargo ${settings.cargoToolchain} run --bin cargo-em-refactor --manifest-path=${settings.refactoringCargoTomlPath} -- ${refactorArgs}`;
}

const isValidBinaryPath = (binaryPath: string): boolean =>
    !!binaryPath && binaryPath.length > 0;
