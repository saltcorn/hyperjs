"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const index_1 = require("./hyperjs-core/index");
// interface ServerListenOptions {
//     backlog: number,
//     exclusive: boolean,
//     host: string,
//     ipv6Only: boolean,
//     reusePort: boolean,
//     path: string,
//     port: number,
//     readableAll: boolean,
//     signal: AbortSignal,
//     writableAll: boolean
// }
// class TestServer {
//   listen(handle, backlog: number, callback);
//   listen(options: ServerListenOptions, callback);
//   listen(path: string, backlog: number, callback);
//   listen(port: number, host: string, backlog: number, callback);
//   listen() {}
// }
function hyperjs() {
    return new index_1.Server();
}
exports.default = hyperjs;
module.exports = Object.assign(hyperjs, { Request: index_1.Request, Response: index_1.Response });
