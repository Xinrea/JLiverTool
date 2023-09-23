/*
 * ATTENTION: The "eval" devtool has been used (maybe by default in mode: "development").
 * This devtool is neither made for production nor for readable output files.
 * It uses "eval()" calls to create a separate source file in the browser devtools.
 * If you are trying to read the output file, select a different devtool (https://webpack.js.org/configuration/devtool/)
 * or disable the default devtool with "devtool: false".
 * If you are looking for production-ready output files, see mode: "production" (https://webpack.js.org/configuration/mode/).
 */
/******/ (() => { // webpackBootstrap
/******/ 	"use strict";
/******/ 	var __webpack_modules__ = ({

/***/ "./src/bililogin.ts":
/*!**************************!*\
  !*** ./src/bililogin.ts ***!
  \**************************/
/***/ ((__unused_webpack_module, exports, __webpack_require__) => {

eval("\nObject.defineProperty(exports, \"__esModule\", ({ value: true }));\nexports.Logout = exports.CheckQrCodeStatus = exports.GetNewQrCode = exports.QrCodeStatus = void 0;\nvar https = __webpack_require__(/*! https */ \"https\");\nvar QrCodeStatus;\n(function (QrCodeStatus) {\n    QrCodeStatus[QrCodeStatus[\"NeedScan\"] = 0] = \"NeedScan\";\n    QrCodeStatus[QrCodeStatus[\"NeedConfirm\"] = 1] = \"NeedConfirm\";\n    QrCodeStatus[QrCodeStatus[\"Success\"] = 2] = \"Success\";\n})(QrCodeStatus = exports.QrCodeStatus || (exports.QrCodeStatus = {}));\nfunction GetNewQrCode() {\n    return new Promise(function (resolve, reject) {\n        https.get('https://passport.bilibili.com/x/passport-login/web/qrcode/generate', function (res) {\n            res.on('data', function (chunk) {\n                var resp = JSON.parse(chunk.toString());\n                // QrCode image is generated from resp['data']['url']\n                // oauthKey is used to check QrCode status\n                resolve({\n                    url: resp['data']['url'],\n                    oauthKey: resp['data']['qrcode_key']\n                });\n            });\n            res.on('error', function (err) {\n                reject(err);\n            });\n        });\n    });\n}\nexports.GetNewQrCode = GetNewQrCode;\nfunction CheckQrCodeStatus(oauthKey) {\n    return new Promise(function (resolve, reject) {\n        var postOptions = {\n            hostname: 'passport.bilibili.com',\n            path: '/x/passport-login/web/qrcode/poll?qrcode_key=' + oauthKey,\n            method: 'GET'\n        };\n        var statusReq = https.request(postOptions, function (res) {\n            var dd = '';\n            res.on('data', function (secCheck) {\n                dd += secCheck;\n            });\n            res.on('end', function () {\n                var resp = JSON.parse(dd);\n                if (resp['data']['code'] === 0) {\n                    var querystring = __webpack_require__(/*! querystring */ \"querystring\");\n                    var url = resp['data']['url'];\n                    var params = querystring.parse(url.split('?')[1]);\n                    resolve({\n                        status: QrCodeStatus.Success,\n                        cookies: params,\n                    });\n                }\n                else {\n                    if (resp['data']['code'] === 86101) {\n                        resolve({\n                            status: QrCodeStatus.NeedScan\n                        });\n                    }\n                    else if (resp['data']['code'] === 86090) {\n                        resolve({\n                            status: QrCodeStatus.NeedConfirm\n                        });\n                    }\n                    else {\n                        reject(resp);\n                    }\n                }\n            });\n            res.on('error', function (err) {\n                reject(err);\n            });\n        });\n        statusReq.end();\n    });\n}\nexports.CheckQrCodeStatus = CheckQrCodeStatus;\nfunction cookiesToString(cookies) {\n    return \"SESSDATA=\" + encodeURIComponent(cookies.SESSDATA) + \"; DedeUserID=\" + cookies.DedeUserID + \"; DedeUserID_ckMd5=\" + cookies.DedeUserID__ckMd5;\n}\nfunction Logout(cookies) {\n    // https://passport.bilibili.com/login/exit/v2\n    return new Promise(function (resolve, reject) {\n        var postData = 'biliCSRF=' + cookies.bili_jct;\n        var postOptions = {\n            hostname: 'passport.bilibili.com',\n            path: '/login/exit/v2',\n            method: 'POST',\n            headers: {\n                'Content-Type': 'application/x-www-form-urlencoded',\n                'Content-Length': Buffer.byteLength(postData),\n                'cookie': cookiesToString(cookies)\n            }\n        };\n        var statusReq = https.request(postOptions, function (res) {\n            var dd = '';\n            res.on('data', function (secCheck) {\n                dd += secCheck;\n            });\n            res.on('end', function () {\n                var resp = JSON.parse(dd);\n                resolve(resp);\n            });\n            res.on('error', function (err) {\n                reject(err);\n            });\n        });\n        statusReq.write(postData);\n        statusReq.end();\n    });\n}\nexports.Logout = Logout;\n\n\n//# sourceURL=webpack://jlivertool/./src/bililogin.ts?");

/***/ }),

/***/ "https":
/*!************************!*\
  !*** external "https" ***!
  \************************/
/***/ ((module) => {

module.exports = require("https");

/***/ }),

/***/ "querystring":
/*!******************************!*\
  !*** external "querystring" ***!
  \******************************/
/***/ ((module) => {

module.exports = require("querystring");

/***/ })

/******/ 	});
/************************************************************************/
/******/ 	// The module cache
/******/ 	var __webpack_module_cache__ = {};
/******/ 	
/******/ 	// The require function
/******/ 	function __webpack_require__(moduleId) {
/******/ 		// Check if module is in cache
/******/ 		var cachedModule = __webpack_module_cache__[moduleId];
/******/ 		if (cachedModule !== undefined) {
/******/ 			return cachedModule.exports;
/******/ 		}
/******/ 		// Create a new module (and put it into the cache)
/******/ 		var module = __webpack_module_cache__[moduleId] = {
/******/ 			// no module.id needed
/******/ 			// no module.loaded needed
/******/ 			exports: {}
/******/ 		};
/******/ 	
/******/ 		// Execute the module function
/******/ 		__webpack_modules__[moduleId](module, module.exports, __webpack_require__);
/******/ 	
/******/ 		// Return the exports of the module
/******/ 		return module.exports;
/******/ 	}
/******/ 	
/************************************************************************/
/******/ 	
/******/ 	// startup
/******/ 	// Load entry module and return exports
/******/ 	// This entry module is referenced by other modules so it can't be inlined
/******/ 	var __webpack_exports__ = __webpack_require__("./src/bililogin.ts");
/******/ 	
/******/ })()
;