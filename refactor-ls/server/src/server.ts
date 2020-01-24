import "reflect-metadata";

import { bootstrapAndReturnConnection } from './services/services';
import { container } from "tsyringe";
import { CodeActionService } from "./services/CodeActionService";
import { ExecuteCommandService } from "./services/ExecuteCommandService";
import { HoverService } from "./services/HoverService";

const connection = bootstrapAndReturnConnection();

connection.onCodeAction(container.resolve(CodeActionService).handleCodeAction);
connection.onExecuteCommand(container.resolve(ExecuteCommandService).handleExecuteCommand);
connection.onHover(container.resolve(HoverService).handleOnHover);