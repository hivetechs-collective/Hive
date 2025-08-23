"use strict";
/**
 * Process Management Type Definitions
 * Unified process management for all process types
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.ProcessEvent = exports.ProcessStatus = exports.ProcessType = void 0;
/**
 * Types of processes we manage
 */
var ProcessType;
(function (ProcessType) {
    ProcessType["SERVICE"] = "service";
    ProcessType["TERMINAL"] = "terminal";
    ProcessType["NODE"] = "node";
    ProcessType["SYSTEM"] = "system"; // System commands
})(ProcessType = exports.ProcessType || (exports.ProcessType = {}));
/**
 * Process status
 */
var ProcessStatus;
(function (ProcessStatus) {
    ProcessStatus["STARTING"] = "starting";
    ProcessStatus["RUNNING"] = "running";
    ProcessStatus["STOPPING"] = "stopping";
    ProcessStatus["STOPPED"] = "stopped";
    ProcessStatus["CRASHED"] = "crashed";
    ProcessStatus["RESTARTING"] = "restarting";
})(ProcessStatus = exports.ProcessStatus || (exports.ProcessStatus = {}));
/**
 * Process events
 */
var ProcessEvent;
(function (ProcessEvent) {
    ProcessEvent["STARTED"] = "process:started";
    ProcessEvent["STOPPED"] = "process:stopped";
    ProcessEvent["CRASHED"] = "process:crashed";
    ProcessEvent["RESTARTED"] = "process:restarted";
    ProcessEvent["OUTPUT"] = "process:output";
    ProcessEvent["ERROR"] = "process:error";
    ProcessEvent["EXIT"] = "process:exit";
})(ProcessEvent = exports.ProcessEvent || (exports.ProcessEvent = {}));
//# sourceMappingURL=process.js.map