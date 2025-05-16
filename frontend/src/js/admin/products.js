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
    var logout_btn, search_btn, create_btn;
    return __generator(this, function (_a) {
        switch (_a.label) {
            case 0: return [4, check_admin_auth()];
            case 1:
                if (!(_a.sent())) {
                    window.location.replace("/index.html");
                    return [2];
                }
                logout_btn = document.getElementById("logout");
                logout_btn.addEventListener("click", admin_logout);
                search_btn = document.getElementById("search_products");
                search_btn.addEventListener("click", load_and_render_products);
                create_btn = document.getElementById("create_product");
                create_btn.addEventListener("click", function () { return __awaiter(void 0, void 0, void 0, function () {
                    return __generator(this, function (_a) {
                        window.location.replace("/admin/product.html");
                        return [2];
                    });
                }); });
                return [4, load_and_render_products()];
            case 2:
                _a.sent();
                return [2];
        }
    });
}); });
function check_admin_auth() {
    return __awaiter(this, void 0, void 0, function () {
        var res;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0: return [4, fetch_csrf("/api/auth/check/admin")];
                case 1:
                    res = _a.sent();
                    return [2, res.ok];
            }
        });
    });
}
function admin_logout() {
    return __awaiter(this, void 0, void 0, function () {
        var res, error_1;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    _a.trys.push([0, 2, , 3]);
                    return [4, fetch_csrf("/api/auth", { method: "DELETE" })];
                case 1:
                    res = _a.sent();
                    if (!res.ok) {
                        console.error("Failed to logout");
                    }
                    else {
                        window.location.replace("/");
                    }
                    return [3, 3];
                case 2:
                    error_1 = _a.sent();
                    console.error(error_1);
                    return [3, 3];
                case 3: return [2];
            }
        });
    });
}
function load_and_render_products() {
    return __awaiter(this, void 0, void 0, function () {
        var name_search_input, name_search, url, res, product_response, error_2;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    name_search_input = document.getElementById("name_search");
                    name_search = name_search_input.value.trim();
                    url = "/api/products";
                    if (name_search) {
                        url += "?name=".concat(encodeURIComponent(name_search));
                    }
                    _a.label = 1;
                case 1:
                    _a.trys.push([1, 4, , 5]);
                    return [4, fetch_csrf(url)];
                case 2:
                    res = _a.sent();
                    if (!res.ok)
                        throw new Error("Failed to fetch products");
                    return [4, res.json()];
                case 3:
                    product_response = _a.sent();
                    render_products(product_response.products);
                    return [3, 5];
                case 4:
                    error_2 = _a.sent();
                    console.error(error_2);
                    return [3, 5];
                case 5: return [2];
            }
        });
    });
}
function render_products(products) {
    var products_list = document.getElementById("products_list");
    products_list.innerHTML = "";
    if (products.length === 0) {
        products_list.innerHTML = "<div class=\"alert alert-info\">No products found.</div>";
        return;
    }
    products.forEach(function (product) {
        var card = document.createElement("div");
        card.className = "col";
        var carousel_html = "";
        if (product.images && product.images.length > 0) {
            var carousel_id_1 = "carousel-".concat(product.id);
            var carousel_indicators = product.images
                .map(function (_, index) { return "\n            <button type=\"button\" data-bs-target=\"#".concat(carousel_id_1, "\" data-bs-slide-to=\"").concat(index, "\" ").concat(index === 0 ? 'class="active" aria-current="true"' : "", " aria-label=\"Slide ").concat(index + 1, "\"></button>\n          "); })
                .join("");
            var carousel_inner = product.images
                .map(function (image, index) { return "\n            <div class=\"carousel-item ".concat(index === 0 ? "active" : "", "\">\n              <img src=\"").concat(image, "\" class=\"d-block w-100\" style=\"height:350px; object-fit: contain;\">\n            </div>\n          "); })
                .join("");
            carousel_html = "\n        <div id=\"".concat(carousel_id_1, "\" class=\"carousel slide\" data-bs-ride=\"carousel\">\n          <div class=\"carousel-indicators\">\n            ").concat(carousel_indicators, "\n          </div>\n          <div class=\"carousel-inner\">\n            ").concat(carousel_inner, "\n          </div>\n          <button class=\"carousel-control-prev\" type=\"button\" data-bs-target=\"#").concat(carousel_id_1, "\" data-bs-slide=\"prev\">\n            <span class=\"carousel-control-prev-icon\" aria-hidden=\"true\"></span>\n            <span class=\"visually-hidden\">Previous</span>\n          </button>\n          <button class=\"carousel-control-next\" type=\"button\" data-bs-target=\"#").concat(carousel_id_1, "\" data-bs-slide=\"next\">\n            <span class=\"carousel-control-next-icon\" aria-hidden=\"true\"></span>\n            <span class=\"visually-hidden\">Next</span>\n          </button>\n        </div>\n      ");
        }
        else {
            carousel_html = "<div class=\"card-img-top bg-secondary text-white d-flex align-items-center justify-content-center\" style=\"height:350px;\">No Image</div>";
        }
        card.innerHTML = "\n      <div class=\"card h-100 shadow-sm d-flex flex-column\">\n        ".concat(carousel_html, "\n        <div class=\"card-body d-flex flex-column justify-content-end\">\n          <span class=\"badge mb-2 ").concat(product.listed ? "bg-success" : "bg-danger", "\">\n            ").concat(product.listed ? "Listed" : "Unlisted", "\n          </span>\n          <h5 class=\"card-title\" id=\"card-name-").concat(product.id, "\"></h5>\n          <p class=\"card-text\">\n            <span id=\"card-description-").concat(product.id, "\"></span><br>\n            \u00A3").concat((product.price / 100).toFixed(2), "\n          </p>\n          <a class=\"btn btn-primary mt-auto\" href=\"/admin/product.html?product_id=").concat(product.id, "\">Edit</a>\n        </div>\n      </div>\n    ");
        products_list.appendChild(card);
        var card_name_elem = document.getElementById("card-name-".concat(product.id));
        if (card_name_elem) {
            card_name_elem.textContent = product.name;
        }
        var card_desc_elem = document.getElementById("card-description-".concat(product.id));
        if (card_desc_elem) {
            card_desc_elem.textContent = product.description;
        }
    });
}
