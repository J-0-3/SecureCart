interface Product {
    id: string;
    name: string;
    description: string;
    price: number;
    images: string[];
}

interface ProductsResponse {
    products: Product[];
}

async function fetch_products(
    name: string,
    price_min: number,
    price_max: number,
): Promise<Product[]> {
    const response = await fetch_csrf(
        `/api/products?name=${encodeURIComponent(name)}&price_min=${price_min}` +
        (price_max === 0 ? "" : `&price_max=${price_max}`),
    );
    if (!response.ok) {
        console.error("Failed to fetch products", response.status);
        return [];
    }
    const products_response: ProductsResponse = await response.json();
    return products_response.products;
}

function store_render_products(products: Product[]) {
    const product_list = document.getElementById("product-list")!;
    product_list.innerHTML = "";

    products.forEach((product) => {
        if (product.images.length === 0) return;

        const carousel_id = `carousel-${product.id}`;
        const carouselIndicators = product.images
            .map(
                (_, index) => `
            <button type="button" data-bs-target="#${carousel_id}" data-bs-slide-to="${index}"
                class="${index === 0 ? "active" : ""}" aria-current="${index === 0 ? "true" : "false"}"
                aria-label="Slide ${index + 1}">
            </button>
        `,
            )
            .join("");

        const carouselInner = product.images
            .map(
                (image, index) => `
            <div class="carousel-item ${index === 0 ? "active" : ""}">
                <img src="${image}" class="d-block w-100""
                    style="height: 350px; object-fit: contain;">
            </div>
        `,
            )
            .join("");

        const card = document.createElement("div");
        card.className = "col";
        card.innerHTML = `
            <div class="card h-100 shadow-sm d-flex flex-column">
                <div id="${carousel_id}" class="carousel slide" data-bs-ride="carousel">
                    <div class="carousel-indicators">
                        ${carouselIndicators}
                    </div>
                    <div class="carousel-inner">
                        ${carouselInner}
                    </div>
                    <button class="carousel-control-prev" type="button" data-bs-target="#${carousel_id}" data-bs-slide="prev">
                        <span class="carousel-control-prev-icon" aria-hidden="true"></span>
                        <span class="visually-hidden">Previous</span>
                    </button>
                    <button class="carousel-control-next" type="button" data-bs-target="#${carousel_id}" data-bs-slide="next">
                        <span class="carousel-control-next-icon" aria-hidden="true"></span>
                        <span class="visually-hidden">Next</span>
                    </button>
                </div>
                <div class="card-body d-flex flex-column justify-content-end">
                    <h5 class="card-title" id="card-name-${product.id}"></h5>
                    <p class="card-text"><span id="card-description-${product.id}"></span><br>£${(product.price / 100).toFixed(2)}</p>
                    <button class="btn btn-primary mt-auto" id="cart-button-${product.id}">Add to Cart</button>
                </div>
            </div>
        `;
        product_list.appendChild(card);
        document.getElementById(`card-name-${product.id}`)!.textContent =
            product.name;
        document.getElementById(`card-description-${product.id}`)!.textContent =
            product.description;

        const cart_button = document.getElementById(`cart-button-${product.id}`)!;
        cart_button.addEventListener("click", () => add_to_cart(product.id));
    });
}

function update_cart_counter(amount: number) {
    const counter = document.getElementById("cart-counter")!;
    counter.textContent = `${amount}`;
}
function add_to_cart(product_id: string) {
    const current_cart: string[] = JSON.parse(
        localStorage.getItem("cart") ?? "[]",
    );
    current_cart.push(product_id);
    localStorage.setItem("cart", JSON.stringify(current_cart));
    update_cart_counter(current_cart.length);
}

async function update_products() {
    const search_input = document.getElementById("search") as HTMLInputElement;
    const price_min = Number(
        (document.getElementById("price-min") as HTMLInputElement).value,
    );
    const price_max = Number(
        (document.getElementById("price-max") as HTMLInputElement).value,
    );
    const products = await fetch_products(
        search_input.value,
        price_min,
        price_max,
    );
    store_render_products(products);
}

function update_price_labels() {
    const price_min_input = document.getElementById(
        "price-min",
    ) as HTMLInputElement;
    const price_max_input = document.getElementById(
        "price-max",
    ) as HTMLInputElement;
    document.getElementById("price-min-value")!.textContent =
        `£${(Number(price_min_input.value) / 100).toFixed(2)}`;
    document.getElementById("price-max-value")!.textContent =
        `£${(Number(price_max_input.value) / 100).toFixed(2)}`;
}

async function logout() {
    const response = await fetch_csrf("/api/auth", {
        method: "DELETE",
    });
    if (!response.ok) {
        console.error("Failed to logout");
        return;
    }
    window.location.replace("/");
}

document.addEventListener("DOMContentLoaded", async () => {
    await update_products();
    document.getElementById("search")!.addEventListener("keydown", (event) => {
        if (event.key === "Enter") {
            event.preventDefault();
            update_products();
        }
    });
    document
        .getElementById("price-min")!
        .addEventListener("input", update_price_labels);
    document
        .getElementById("price-max")!
        .addEventListener("input", update_price_labels);
    document
        .getElementById("price-min")!
        .addEventListener("change", update_products);
    document
        .getElementById("price-max")!
        .addEventListener("change", update_products);
    update_price_labels();
    document.getElementById("logout")!.addEventListener("click", logout);
    const cart = JSON.parse(localStorage.getItem("cart") ?? "[]");
    update_cart_counter(cart.length);
});
