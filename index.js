"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const index_1 = require("./hyperjs-core/index");
const hyper_js_1 = require("./src-ts/hyper_js");
function hyperjs() {
    return new hyper_js_1.HyperJs();
}
exports.default = hyperjs;
module.exports = Object.assign(hyperjs, { Request: index_1.Request, Response: index_1.Response });
