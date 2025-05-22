interface UserData {
    id: string;
    email: string;
    forename: string;
    surname: string;
    address: string;
    role: string;
}

interface Order {
    id: string;
    amount_charged: number;
    user_id: string;
    order_placed: string;
    status: string;
}

let is_admin: boolean = false;
let target_user_id: string | null = null;
let viewing_self: boolean = false;

document.addEventListener("DOMContentLoaded", init_user_page);

async function init_user_page(): Promise<void> {
    try {
        const res = await fetch("/api/auth/check/admin");
        is_admin = res.ok;
    } catch (err) {
        console.error("Admin check error:", err);
        is_admin = false;
    }

    const url_params = new URLSearchParams(window.location.search);
    const query_user_id = url_params.get("user_id");

    if (is_admin && query_user_id) {
        target_user_id = query_user_id;
    } else {
        target_user_id = null;
    }

    viewing_self = target_user_id === null;

    if (!is_admin || viewing_self) {
        document.getElementById("password_section")!.style.display = "block";
        document.getElementById("2fa_section")!.style.display = "block";
    }

    const user_url = target_user_id
        ? `/api/users/${target_user_id}`
        : `/api/users/self`;

    try {
        const res = await fetch_csrf(user_url);
        if (!res.ok) throw new Error("Failed to fetch user data");
        const user: UserData = await res.json();
        populate_form(user);
    } catch (err) {
        console.error(err);
        show_user_page_message_modal("Error fetching user data");
    }

    document.getElementById("user_form")!.addEventListener("submit", update_user);
    document
        .getElementById("delete_user")!
        .addEventListener("click", delete_user);
    document
        .getElementById("promote_user")!
        .addEventListener("click", promote_user);
    document.getElementById("2fa-button")!.addEventListener("click", async () => {
        window.location.replace("/enroll2fa.html");
    });
}

function populate_form(user: UserData): void {
    (document.getElementById("email") as HTMLInputElement).value = user.email;
    (document.getElementById("forename") as HTMLInputElement).value =
        user.forename;
    (document.getElementById("surname") as HTMLInputElement).value = user.surname;
    (document.getElementById("address") as HTMLTextAreaElement).value =
        user.address;
    (document.getElementById("role_display") as HTMLElement).textContent =
        user.role;

    if (is_admin && target_user_id && user.role === "Customer") {
        document.getElementById("promote_user")!.style.display = "block";
    }
    if (user.role.toLowerCase() === "customer") {
        load_orders();
    } else {
        document.getElementById("orders_section")!.style.display = "none";
    }
}

async function update_user(event: Event): Promise<void> {
    event.preventDefault();

    const email = (document.getElementById("email") as HTMLInputElement).value;
    const forename = (document.getElementById("forename") as HTMLInputElement)
        .value;
    const surname = (document.getElementById("surname") as HTMLInputElement)
        .value;
    const address = (document.getElementById("address") as HTMLTextAreaElement)
        .value;

    const user_url = target_user_id
        ? `/api/users/${target_user_id}`
        : `/api/users/self`;

    try {
        const res = await fetch_csrf(user_url, {
            method: "PUT",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ email, forename, surname, address }),
        });
        if (!res.ok) throw new Error("Failed to update user details");
        show_user_page_message_modal("User updated successfully");
    } catch (err) {
        console.error(err);
        show_user_page_message_modal("Error updating user");
    }

    if (viewing_self) {
        const password = (document.getElementById("password") as HTMLInputElement)
            .value;
        if (password.trim().length > 0) {
            try {
                const res = await fetch_csrf("/api/users/self/credential", {
                    method: "PUT",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({ Password: { password } }),
                });
                if (!res.ok) throw new Error("Failed to update password");
                show_user_page_message_modal("Password updated successfully");
            } catch (err) {
                console.error(err);
                show_user_page_message_modal("Error updating password");
            }
        }
    }
}

async function delete_user() {
    if (
        !confirm(
            "Are you sure you want to permanently delete this account? This cannot be undone.",
        )
    ) {
        return;
    }

    const user_url = target_user_id
        ? `/api/users/${target_user_id}`
        : `/api/users/self`;

    try {
        const res = await fetch_csrf(user_url, { method: "DELETE" });
        if (!res.ok) throw new Error("Failed to delete user");
        show_user_page_message_modal("User deleted successfully");
        window.location.replace("/");
    } catch (err) {
        console.error(err);
        show_user_page_message_modal("Error deleting user");
    }
}

async function promote_user() {
    if (!target_user_id) return;
    if (
        !confirm(
            "Are you sure you want to promote this user to an admin? It will not be possible to demote them afterwards.",
        )
    ) {
        return;
    }

    try {
        const res = await fetch_csrf(`/api/users/${target_user_id}/promote`, {
            method: "POST",
        });
        if (!res.ok) throw new Error("Failed to promote user");
        show_user_page_message_modal("User promoted to admin");
        setTimeout(() => window.location.reload(), 1000);
    } catch (err) {
        console.error(err);
        show_user_page_message_modal("Error promoting user");
    }
}

function show_user_page_message_modal(message: string): void {
    document.getElementById("alert-modal-body")!.textContent = message;
    const modal = new bootstrap.Modal(document.getElementById("alert-modal")!);
    modal.show();
}

async function load_orders() {
    let orders_url = "";
    if (is_admin && target_user_id) {
        orders_url = `/api/orders?user_id=${target_user_id}`;
    } else {
        orders_url = `/api/orders`;
    }

    try {
        const res = await fetch_csrf(orders_url);
        if (!res.ok) throw new Error("Failed to fetch orders");
        const order_res: OrderResponse = await res.json();
        const orders: Order[] = order_res.orders;
        const orders_list = document.getElementById("orders_list");
        if (!orders_list) return;
        orders_list.innerHTML = "";

        if (orders.length === 0) {
            const li = document.createElement("li");
            li.className = "list-group-item";
            li.textContent = "No orders found.";
            orders_list.appendChild(li);
            return;
        }

        orders.forEach((order) => {
            const li = document.createElement("li");
            li.className = "list-group-item list-group-item-action";
            const charged_amount_pounds = (order.amount_charged / 100).toFixed(2);
            const order_id_label = document.createElement("strong");
            order_id_label.textContent = "Order ID: ";
            const amount_label = document.createElement("strong");
            amount_label.textContent = "Amount: ";
            const placed_label = document.createElement("strong");
            placed_label.textContent = "Placed: ";
            const status_label = document.createElement("strong");
            status_label.textContent = "Status: ";
            li.append(
                order_id_label,
                `${order.id} - `,
                amount_label,
                `£${charged_amount_pounds} - `,
                placed_label,
                `${new Date(order.order_placed).toLocaleString()} - `,
                status_label,
                order.status
            );
            li.style.cursor = "pointer";
            li.addEventListener("click", () => {
                window.location.replace(`/order.html?order=${order.id}`);
            });
            orders_list.appendChild(li);
        });
    } catch (error) {
        console.error("Error loading orders:", error);
        const ordersList = document.getElementById("orders_list");
        if (ordersList) {
            ordersList.innerHTML =
                '<li class="list-group-item text-danger">Error loading orders.</li>';
        }
    }
}
