import "reflect-metadata";

import { bootstrapAndReturnConnection } from './services/services';
import { container } from "tsyringe";
import { ExecuteCommandService } from "./services/ExecuteCommandService";
import { HoverService } from "./services/HoverService";

const connection = bootstrapAndReturnConnection();

connection.onExecuteCommand(container.resolve(ExecuteCommandService).handleExecuteCommand);
connection.onHover(container.resolve(HoverService).handleOnHover);