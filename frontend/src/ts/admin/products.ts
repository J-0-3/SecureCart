interface Product {
    id: string;
    name: string;
    description: string;
    price: number; // stored in pennies
    listed: boolean;
    images: string[];
}

interface ProductResponse {
    products: Product[];
}

document.addEventListener("DOMContentLoaded", async () => {
    if (!(await check_admin_auth())) {
        window.location.replace("/index.html");
        return;
    }

    const logout_btn = document.getElementById("logout") as HTMLButtonElement;
    logout_btn.addEventListener("click", admin_logout);

    const search_btn = document.getElementById(
        "search_products",
    ) as HTMLButtonElement;
    search_btn.addEventListener("click", load_and_render_products);

    const create_btn = document.getElementById(
        "create_product",
    )! as HTMLButtonElement;
    create_btn.addEventListener("click", async () => {
        window.location.replace("/admin/product.html");
    });

    await load_and_render_products();
});

async function check_admin_auth(): Promise<boolean> {
    const res = await fetch_csrf("/api/auth/check/admin");
    return res.ok;
}

async function admin_logout(): Promise<void> {
    try {
        const res = await fetch_csrf("/api/auth", { method: "DELETE" });
        if (!res.ok) {
            console.error("Failed to logout");
        } else {
            window.location.replace("/");
        }
    } catch (error) {
        console.error(error);
    }
}

async function load_and_render_products(): Promise<void> {
    const name_search_input = document.getElementById(
        "name_search",
    ) as HTMLInputElement;
    const name_search = name_search_input.value.trim();

    let url = "/api/products";
    if (name_search) {
        url += `?name=${encodeURIComponent(name_search)}`;
    }

    try {
        const res = await fetch_csrf(url);
        if (!res.ok) throw new Error("Failed to fetch products");
        const product_response: ProductResponse = await res.json();
        render_products(product_response.products);
    } catch (error) {
        console.error(error);
    }
}

function render_products(products: Product[]): void {
    const products_list = document.getElementById("products_list")!;
    products_list.innerHTML = "";

    if (products.length === 0) {
        products_list.innerHTML = `<div class="alert alert-info">No products found.</div>`;
        return;
    }

    products.forEach((product) => {
        const card = document.createElement("div");
        card.className = "col";

        let carousel_html = "";
        if (product.images && product.images.length > 0) {
            const carousel_id = `carousel-${product.id}`;
            const carousel_indicators = product.images
                .map(
                    (_, index) => `
            <button type="button" data-bs-target="#${carousel_id}" data-bs-slide-to="${index}" ${index === 0 ? 'class="active" aria-current="true"' : ""
                        } aria-label="Slide ${index + 1}"></button>
          `,
                )
                .join("");
            const carousel_inner = product.images
                .map(
                    (image, index) => `
            <div class="carousel-item ${index === 0 ? "active" : ""}">
              <img src="${image}" class="d-block w-100" style="height:350px; object-fit: contain;">
            </div>
          `,
                )
                .join("");
            carousel_html = `
        <div id="${carousel_id}" class="carousel slide" data-bs-ride="carousel">
          <div class="carousel-indicators">
            ${carousel_indicators}
          </div>
          <div class="carousel-inner">
            ${carousel_inner}
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
      `;
        } else {
            carousel_html = `<div class="card-img-top bg-secondary text-white d-flex align-items-center justify-content-center" style="height:350px;">No Image</div>`;
        }

        card.innerHTML = `
      <div class="card h-100 shadow-sm d-flex flex-column">
        ${carousel_html}
        <div class="card-body d-flex flex-column justify-content-end">
          <span class="badge mb-2 ${product.listed ? "bg-success" : "bg-danger"}">
            ${product.listed ? "Listed" : "Unlisted"}
          </span>
          <h5 class="card-title" id="card-name-${product.id}"></h5>
          <p class="card-text">
            <span id="card-description-${product.id}"></span><br>
            £${(product.price / 100).toFixed(2)}
          </p>
          <a class="btn btn-primary mt-auto" href="/admin/product.html?product_id=${product.id}">Edit</a>
        </div>
      </div>
    `;
        products_list.appendChild(card);

        const card_name_elem = document.getElementById(`card-name-${product.id}`);
        if (card_name_elem) {
            card_name_elem.textContent = product.name;
        }
        const card_desc_elem = document.getElementById(
            `card-description-${product.id}`,
        );
        if (card_desc_elem) {
            card_desc_elem.textContent = product.description;
        }
    });
}
