interface Product {
    id: string;
    name: string;
    description: string;
    price: number;
    listed: boolean;
    images: string[];
}

interface ProductInsert {
    name: string;
    description: string;
    price: number;
    listed: boolean;
}

document.addEventListener("DOMContentLoaded", async () => {
    const url_params = new URLSearchParams(window.location.search);
    const product_id = url_params.get("product_id");

    if (!product_id) {
        handle_create_state();
        return;
    }

    await load_product_details(product_id);

    document
        .getElementById("delete_product")!
        .addEventListener("click", () => delete_product(product_id));
    document
        .getElementById("product_form")!
        .addEventListener("submit", (event) => {
            event.preventDefault();
            update_product(product_id);
        });
    document
        .getElementById("image_upload")!
        .addEventListener("change", (event) => {
            upload_image(product_id, (event.target as HTMLInputElement).files![0]);
        });
});

async function handle_create_state() {
    const create_button = document.getElementById("create_button")!;
    const image_section = document.getElementById("image_section")!;
    const save_button = document.getElementById("save_button")!;

    save_button.style.display = "none";
    create_button.style.display = "inline";

    image_section.querySelector("input")!.disabled = true;
    image_section.hidden = true;

    create_button.addEventListener("click", async () => {
        const name = (document.getElementById("name") as HTMLInputElement).value;
        const description = (
            document.getElementById("description") as HTMLTextAreaElement
        ).value;
        const price =
            parseFloat((document.getElementById("price") as HTMLInputElement).value) *
            100;
        const listed =
            (document.getElementById("listed") as HTMLSelectElement).value === "true";

        const new_product: ProductInsert = {
            name,
            description,
            price,
            listed,
        };

        try {
            const res = await fetch_csrf("/api/products", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify(new_product),
            });

            if (!res.ok) throw new Error("Failed to create product.");
            const created_product: Product = await res.json();
            window.location.replace(
                `/admin/product.html?product_id=${created_product.id}`,
            );
        } catch (_) {
            show_alert("Failed to create product.");
        }
    });
}

async function load_product_details(product_id: string) {
    const res = await fetch_csrf(`/api/products/${product_id}`);
    if (!res.ok) throw new Error("Failed to fetch product details.");
    const product: Product = await res.json();

    (document.getElementById("name")! as HTMLInputElement).value = product.name;
    (document.getElementById("description")! as HTMLInputElement).value =
        product.description;
    (document.getElementById("price")! as HTMLInputElement).value = (
        product.price / 100
    ).toFixed(2);
    (document.getElementById("listed")! as HTMLInputElement).value =
        product.listed ? "true" : "false";

    const image_list = document.getElementById("image_list")!;
    image_list.innerHTML = "";
    product.images.forEach((image_url) => {
        const img = document.createElement("img");
        img.src = image_url;
        img.className = "img-thumbnail";
        img.style.width = "150px";
        img.style.margin = "5px";
        image_list!.appendChild(img);
    });
}

async function update_product(product_id: string) {
    const name = (document.getElementById("name") as HTMLInputElement).value;
    const description = (
        document.getElementById("description") as HTMLTextAreaElement
    ).value;
    const price =
        parseFloat((document.getElementById("price") as HTMLInputElement).value) *
        100; // in pennies
    const listed =
        (document.getElementById("listed") as HTMLSelectElement).value === "true";

    try {
        const res = await fetch_csrf(`/api/products/${product_id}`, {
            method: "PUT",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                name,
                description,
                price,
                listed,
            }),
        });

        if (!res.ok) throw new Error("Failed to update product.");
        show_alert("Product updated successfully.");
    } catch (_) {
        show_alert("Failed to update product.");
    }
}

async function upload_image(product_id: string, file: File) {
    const form_data = new FormData();
    form_data.append("image", file);

    try {
        const res = await fetch_csrf(`/api/products/${product_id}/images`, {
            method: "POST",
            body: form_data,
        });

        if (!res.ok) throw new Error("Failed to upload image.");
        show_alert("Image uploaded successfully.");
        await load_product_details(product_id);
    } catch (_) {
        show_alert("Failed to upload image.");
    }
}

async function delete_product(product_id: string) {
    try {
        const res = await fetch_csrf(`/api/products/${product_id}`, {
            method: "DELETE",
        });

        if (!res.ok) throw new Error("Failed to delete product.");
        show_alert("Product deleted successfully.");
        window.location.replace("/admin/products.html");
    } catch (_) {
        show_alert("Failed to delete product.");
    }
}

function show_alert(message: string) {
    document.getElementById("alert-modal-body")!.innerText = message;
    const modal = new bootstrap.Modal(document.getElementById("alert-modal")!);
    modal.show();
}
