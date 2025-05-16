"use strict";
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __generator = (this && this.__generator) || function (thisArg, body) {
    var _ = { label: 0, sent: function() { if (t[0] & 1) throw t[1]; return t[1]; }, trys: [], ops: [] }, f, y, t, g = Object.create((typeof Iterator === "function" ? Iterator : Object).prototype);
    return g.next = verb(0), g["throw"] = verb(1), g["return"] = verb(2), typeof Symbol === "function" && (g[Symbol.iterator] = function() { return this; }), g;
    function verb(n) { return function (v) { return step([n, v]); }; }
    function step(op) {
        if (f) throw new TypeError("Generator is already executing.");
        while (g && (g = 0, op[0] && (_ = 0)), _) try {
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
document.addEventListener("DOMContentLoaded", function () { return __awaiter(void 0, void 0, void 0, function () {
    return __generator(this, function (_a) {
        switch (_a.label) {
            case 0: return [4, fetch_csrf("/api/auth/check/admin")];
            case 1:
                if (!(_a.sent()).ok) {
                    window.location.replace("/index.html");
                    return [2];
                }
                document
                    .getElementById("logout")
                    .addEventListener("click", users_page_logout);
                document
                    .getElementById("search_users")
                    .addEventListener("click", load_and_render_users);
                load_and_render_users();
                return [2];
        }
    });
}); });
function load_and_render_users() {
    return __awaiter(this, void 0, void 0, function () {
        var role, email, url, params, res, user_response, users, err_1;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    role = document.getElementById("role_filter")
                        .value;
                    email = document.getElementById("email_search")
                        .value;
                    url = "/api/users";
                    params = [];
                    if (role) {
                        params.push("role=".concat(encodeURIComponent(role)));
                    }
                    if (email) {
                        params.push("email=".concat(encodeURIComponent(email)));
                    }
                    if (params.length > 0) {
                        url += "?".concat(params.join("&"));
                    }
                    _a.label = 1;
                case 1:
                    _a.trys.push([1, 4, , 5]);
                    return [4, fetch_csrf(url)];
                case 2:
                    res = _a.sent();
                    if (!res.ok)
                        throw new Error("Failed to fetch users");
                    return [4, res.json()];
                case 3:
                    user_response = _a.sent();
                    users = user_response.users;
                    render_users(users);
                    return [3, 5];
                case 4:
                    err_1 = _a.sent();
                    console.error(err_1);
                    return [3, 5];
                case 5: return [2];
            }
        });
    });
}
function render_users(users) {
    var users_list = document.getElementById("users_list");
    users_list.innerHTML = users.length
        ? ""
        : "<div class=\"alert alert-info\">No users found with current criteria.</div>";
    users.forEach(function (user) {
        var user_item = document.createElement("a");
        user_item.href = "/user.html?user_id=".concat(user.id);
        user_item.className = "list-group-item list-group-item-action";
        user_item.innerHTML = "\n      <div class=\"d-flex w-100 justify-content-between\">\n        <h5 class=\"mb-1\" id=\"user-".concat(user.id, "-name\"></h5>\n        <small><span id=\"user-").concat(user.id, "-email\"></span></small>\n      </div>\n      <p class=\"mb-1\">Role: ").concat(user.role, "</p>\n      <small>Address: <span id=\"user-").concat(user.id, "-address\"></span></small>\n    ");
        users_list.appendChild(user_item);
        document.getElementById("user-".concat(user.id, "-name")).textContent =
            "".concat(user.forename, " ").concat(user.surname);
        document.getElementById("user-".concat(user.id, "-email")).textContent = user.email;
        document.getElementById("user-".concat(user.id, "-address")).textContent =
            user.address;
    });
}
function users_page_logout() {
    return __awaiter(this, void 0, void 0, function () {
        var res, err_2;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    _a.trys.push([0, 2, , 3]);
                    return [4, fetch_csrf("/api/auth", { method: "DELETE" })];
                case 1:
                    res = _a.sent();
                    if (!res.ok)
                        console.error("Failed to logout");
                    else
                        window.location.replace("/");
                    return [3, 3];
                case 2:
                    err_2 = _a.sent();
                    console.error(err_2);
                    return [3, 3];
                case 3: return [2];
            }
        });
    });
}
