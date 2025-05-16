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
    var url_params, product_id;
    return __generator(this, function (_a) {
        switch (_a.label) {
            case 0:
                url_params = new URLSearchParams(window.location.search);
                product_id = url_params.get("product_id");
                if (!product_id) {
                    handle_create_state();
                    return [2];
                }
                return [4, load_product_details(product_id)];
            case 1:
                _a.sent();
                document
                    .getElementById("delete_product")
                    .addEventListener("click", function () { return delete_product(product_id); });
                document
                    .getElementById("product_form")
                    .addEventListener("submit", function (event) {
                    event.preventDefault();
                    update_product(product_id);
                });
                document
                    .getElementById("image_upload")
                    .addEventListener("change", function (event) {
                    upload_image(product_id, event.target.files[0]);
                });
                return [2];
        }
    });
}); });
function handle_create_state() {
    return __awaiter(this, void 0, void 0, function () {
        var create_button, image_section, save_button;
        var _this = this;
        return __generator(this, function (_a) {
            create_button = document.getElementById("create_button");
            image_section = document.getElementById("image_section");
            save_button = document.getElementById("save_button");
            save_button.style.display = "none";
            create_button.style.display = "inline";
            image_section.querySelector("input").disabled = true;
            image_section.hidden = true;
            create_button.addEventListener("click", function () { return __awaiter(_this, void 0, void 0, function () {
                var name, description, price, listed, new_product, res, created_product, _1;
                return __generator(this, function (_a) {
                    switch (_a.label) {
                        case 0:
                            name = document.getElementById("name").value;
                            description = document.getElementById("description").value;
                            price = parseFloat(document.getElementById("price").value) *
                                100;
                            listed = document.getElementById("listed").value === "true";
                            new_product = {
                                name: name,
                                description: description,
                                price: price,
                                listed: listed,
                            };
                            _a.label = 1;
                        case 1:
                            _a.trys.push([1, 4, , 5]);
                            return [4, fetch_csrf("/api/products", {
                                    method: "POST",
                                    headers: {
                                        "Content-Type": "application/json",
                                    },
                                    body: JSON.stringify(new_product),
                                })];
                        case 2:
                            res = _a.sent();
                            if (!res.ok)
                                throw new Error("Failed to create product.");
                            return [4, res.json()];
                        case 3:
                            created_product = _a.sent();
                            window.location.replace("/admin/product.html?product_id=".concat(created_product.id));
                            return [3, 5];
                        case 4:
                            _1 = _a.sent();
                            show_alert("Failed to create product.");
                            return [3, 5];
                        case 5: return [2];
                    }
                });
            }); });
            return [2];
        });
    });
}
function load_product_details(product_id) {
    return __awaiter(this, void 0, void 0, function () {
        var res, product, image_list;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0: return [4, fetch_csrf("/api/products/".concat(product_id))];
                case 1:
                    res = _a.sent();
                    if (!res.ok)
                        throw new Error("Failed to fetch product details.");
                    return [4, res.json()];
                case 2:
                    product = _a.sent();
                    document.getElementById("name").value = product.name;
                    document.getElementById("description").value =
                        product.description;
                    document.getElementById("price").value = (product.price / 100).toFixed(2);
                    document.getElementById("listed").value =
                        product.listed ? "true" : "false";
                    image_list = document.getElementById("image_list");
                    image_list.innerHTML = "";
                    product.images.forEach(function (image_url) {
                        var img = document.createElement("img");
                        img.src = image_url;
                        img.className = "img-thumbnail";
                        img.style.width = "150px";
                        img.style.margin = "5px";
                        image_list.appendChild(img);
                    });
                    return [2];
            }
        });
    });
}
function update_product(product_id) {
    return __awaiter(this, void 0, void 0, function () {
        var name, description, price, listed, res, _2;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    name = document.getElementById("name").value;
                    description = document.getElementById("description").value;
                    price = parseFloat(document.getElementById("price").value) *
                        100;
                    listed = document.getElementById("listed").value === "true";
                    _a.label = 1;
                case 1:
                    _a.trys.push([1, 3, , 4]);
                    return [4, fetch_csrf("/api/products/".concat(product_id), {
                            method: "PUT",
                            headers: {
                                "Content-Type": "application/json",
                            },
                            body: JSON.stringify({
                                name: name,
                                description: description,
                                price: price,
                                listed: listed,
                            }),
                        })];
                case 2:
                    res = _a.sent();
                    if (!res.ok)
                        throw new Error("Failed to update product.");
                    show_alert("Product updated successfully.");
                    return [3, 4];
                case 3:
                    _2 = _a.sent();
                    show_alert("Failed to update product.");
                    return [3, 4];
                case 4: return [2];
            }
        });
    });
}
function upload_image(product_id, file) {
    return __awaiter(this, void 0, void 0, function () {
        var form_data, res, _3;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    form_data = new FormData();
                    form_data.append("image", file);
                    _a.label = 1;
                case 1:
                    _a.trys.push([1, 4, , 5]);
                    return [4, fetch_csrf("/api/products/".concat(product_id, "/images"), {
                            method: "POST",
                            body: form_data,
                        })];
                case 2:
                    res = _a.sent();
                    if (!res.ok)
                        throw new Error("Failed to upload image.");
                    show_alert("Image uploaded successfully.");
                    return [4, load_product_details(product_id)];
                case 3:
                    _a.sent();
                    return [3, 5];
                case 4:
                    _3 = _a.sent();
                    show_alert("Failed to upload image.");
                    return [3, 5];
                case 5: return [2];
            }
        });
    });
}
function delete_product(product_id) {
    return __awaiter(this, void 0, void 0, function () {
        var res, _4;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    _a.trys.push([0, 2, , 3]);
                    return [4, fetch_csrf("/api/products/".concat(product_id), {
                            method: "DELETE",
                        })];
                case 1:
                    res = _a.sent();
                    if (!res.ok)
                        throw new Error("Failed to delete product.");
                    show_alert("Product deleted successfully.");
                    window.location.replace("/admin/products.html");
                    return [3, 3];
                case 2:
                    _4 = _a.sent();
                    show_alert("Failed to delete product.");
                    return [3, 3];
                case 3: return [2];
            }
        });
    });
}
function show_alert(message) {
    document.getElementById("alert-modal-body").innerText = message;
    var modal = new bootstrap.Modal(document.getElementById("alert-modal"));
    modal.show();
}
