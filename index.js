"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const index_1 = require("./hyperjs-core/index");
class Server {
    listen() {
        const args = Array.prototype.slice.call(arguments);
        const done = typeof args[args.length - 1] === 'function' ? args[args.length - 1] : null;
        let rsServer = new index_1.Server();
        // (port, hostname, backlog[, callback])
        if (typeof args[0] === 'number' && typeof args[1] === 'string' && typeof args[2] === 'number') {
            const options = {
                port: args[0],
                host: args[1],
                backlog: args[2],
            };
            rsServer.listenTcp(options, done);
        }
        // (port, hostname[, callback])
        else if (typeof args[0] === 'number' && typeof args[1] === 'string') {
            const options = {
                port: args[0],
                host: args[1],
            };
            rsServer.listenTcp(options, done);
        }
        // (port[, callback])
        else if (typeof args[0] === 'number') {
            const options = {
                port: args[0],
            };
            rsServer.listenTcp(options, done);
        }
        // (path[, callback])
        else if (typeof args[0] === 'string') {
            const options = {
                path: args[0],
            };
            rsServer.listenIpc(options, done);
        }
        // ([callback])
        else if (done) {
            rsServer.listenTcp({}, done);
        }
        // (handle, listeningListener)
        else {
            throw new Error('Listening on handle is not supported in this implementation');
        }
        return rsServer;
    }
}
function hyperjs() {
    return new Server();
}
exports.default = hyperjs;
module.exports = Object.assign(hyperjs, { Request: index_1.Request, Response: index_1.Response });
