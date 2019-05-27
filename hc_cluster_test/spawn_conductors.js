"use strict";
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : new P(function (resolve) { resolve(result.value); }).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __generator = (this && this.__generator) || function (thisArg, body) {
    var _ = { label: 0, sent: function() { if (t[0] & 1) throw t[1]; return t[1]; }, trys: [], ops: [] }, f, y, t, g;
    return g = { next: verb(0), "throw": verb(1), "return": verb(2) }, typeof Symbol === "function" && (g[Symbol.iterator] = function() { return this; }), g;
    function verb(n) { return function (v) { return step([n, v]); }; }
    function step(op) {
        if (f) throw new TypeError("Generator is already executing.");
        while (_) try {
            if (f = 1, y && (t = op[0] & 2 ? y["return"] : op[0] ? y["throw"] || ((t = y["return"]) && t.call(y), 0) : y.next) && !(t = t.call(y, op[1])).done) return t;
            if (y = 0, t) op = [op[0] & 2, t.value];
            switch (op[0]) {
                case 0: case 1: t = op; break;
                case 4: _.label++; return { value: op[1], done: false };
                case 5: _.label++; y = op[1]; op = [0]; continue;
                case 7: op = _.ops.pop(); _.trys.pop(); continue;
                default:
                    if (!(t = _.trys, t = t.length > 0 && t[t.length - 1]) && (op[0] === 6 || op[0] === 2)) { _ = 0; continue; }
                    if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) { _.label = op[1]; break; }
                    if (op[0] === 6 && _.label < t[1]) { _.label = t[1]; t = op; break; }
                    if (t && _.label < t[2]) { _.label = t[2]; _.ops.push(op); break; }
                    if (t[2]) _.ops.pop();
                    _.trys.pop(); continue;
            }
            op = body.call(thisArg, _);
        } catch (e) { op = [6, e]; y = 0; } finally { f = t = 0; }
        if (op[0] & 5) throw op[1]; return { value: op[0] ? op[1] : void 0, done: true };
    }
};
var _this = this;
exports.__esModule = true;
var child_process = require("child_process");
var fs = require("fs");
var os = require("os");
var path = require("path");
var config_1 = require("./config");
var holochainBin = process.env.EMULATION_HOLOCHAIN_BIN_PATH;
var genConfig = function (index, debugging, tmpPath, n3hPath) {
    var adminPort = 3000 + index;
    var instancePort = 4000 + index;
    var config = "\npersistence_dir = \"" + tmpPath + "\"\nexpose_trace_signals = true\nagents = []\ndnas = []\ninstances = []\n\n[[interfaces]]\nadmin = true\nid = \"" + config_1.adminInterfaceId + "\"\ninstances = []\n    [interfaces.driver]\n    type = \"websocket\"\n    port = " + adminPort + "\n\n[[interfaces]]\nadmin = true\nid = \"" + config_1.dnaInterfaceId + "\"\ninstances = []\n    [interfaces.driver]\n    type = \"websocket\"\n    port = " + instancePort + "\n\n[logger]\ntype = \"debug\"\n" + (debugging ? '' : '[[logger.rules.rules]]') + "\n" + (debugging ? '' : 'exclude = true') + "\n" + (debugging ? '' : 'pattern = "^debug"') + "\n\n[network]\nn3h_log_level = \"" + (debugging ? 'i' : 'e') + "\"\nbootstrap_nodes = []\nn3h_mode = \"REAL\"\nn3h_persistence_path = \"" + n3hPath + "\"\n    ";
    return { config: config, adminPort: adminPort, instancePort: instancePort };
};
var spawnConductor = function (i, debugging) {
    var tmpPath = fs.mkdtempSync(path.join(os.tmpdir(), 'n3h-test-conductors-'));
    var n3hPath = path.join(tmpPath, 'n3h-storage');
    fs.mkdirSync(n3hPath);
    var configPath = path.join(tmpPath, "empty-conductor-" + i + ".toml");
    var _a = genConfig(i, debugging, tmpPath, n3hPath), config = _a.config, adminPort = _a.adminPort, instancePort = _a.instancePort;
    fs.writeFileSync(configPath, config);
    console.info("Spawning conductor" + i + " process...");
    var handle = child_process.spawn(holochainBin, ['-c', configPath]);
    handle.stdout.on('data', function (data) { return console.log("[C" + i + "]", data.toString('utf8')); });
    handle.stderr.on('data', function (data) { return console.error("!C" + i + "!", data.toString('utf8')); });
    handle.on('close', function (code) { return console.log("conductor " + i + " exited with code", code); });
    console.info("Conductor" + i + " process spawning successful");
    return new Promise(function (resolve) {
        handle.stdout.on('data', function (data) {
            // wait for the logs to convey that the interfaces have started
            // because the consumer of this function needs those interfaces
            // to be started so that it can initiate, and form,
            // the websocket connections
            if (data.toString('utf8').indexOf('Starting interfaces...') >= 0) {
                resolve({
                    adminPort: adminPort,
                    instancePort: instancePort,
                    handle: handle
                });
            }
        });
    });
};
var spawnConductors = function (numberOfConductors, debugging) { return __awaiter(_this, void 0, void 0, function () {
    var promises, firstConductor, i;
    return __generator(this, function (_a) {
        switch (_a.label) {
            case 0:
                promises = [];
                return [4 /*yield*/, spawnConductor(0, debugging)];
            case 1:
                firstConductor = _a.sent();
                promises.push(firstConductor);
                for (i = 1; i < numberOfConductors; i++) {
                    promises.push(spawnConductor(i, debugging));
                }
                return [2 /*return*/, Promise.all(promises)];
        }
    });
}); };
exports["default"] = spawnConductors;
