interface Order {
    id: string;
    amount_charged: number;
    order_placed: string;
    user_id: string;
    status: string;
}

interface OrderResponse {
    orders: Order[];
}

document.addEventListener("DOMContentLoaded", async () => {
    if (!(await fetch_csrf("/api/auth/check/admin")).ok) {
        window.location.replace("/index.html");
        return;
    }
    document
        .getElementById("logout")!
        .addEventListener("click", orders_page_logout);
    document
        .getElementById("search_orders")!
        .addEventListener("click", load_and_render_orders);
    load_and_render_orders();
});

async function load_and_render_orders() {
    const status = (document.getElementById("status_filter") as HTMLSelectElement)
        .value;
    let url = "/api/orders";
    if (status) {
        url += `?status=${encodeURIComponent(status)}`;
    }
    try {
        const res = await fetch_csrf(url);
        if (!res.ok) throw new Error("Failed to fetch orders");
        const order_response: OrderResponse = await res.json();
        const orders = order_response.orders;
        orders.sort(
            (a, b) =>
                new Date(b.order_placed).getTime() - new Date(a.order_placed).getTime(),
        );
        render_orders(orders);
    } catch (err) {
        console.error(err);
    }
}

function render_orders(orders: Order[]) {
    const orders_list = document.getElementById("orders_list");
    if (!orders_list) return;
    orders_list.innerHTML = orders.length
        ? ""
        : `<div class="alert alert-info">No orders found with current criteria.</div>`;
    orders.forEach((order) => {
        const order_item = document.createElement("a");
        order_item.href = `/order.html?order=${order.id}`;
        order_item.className = "list-group-item list-group-item-action";
        order_item.innerHTML = `
      <div class="d-flex w-100 justify-content-between">
        <h5 class="mb-1">Order #${order.id}</h5>
        <small>${new Date(order.order_placed).toLocaleString()}</small>
      </div>
      <p class="mb-1">Status: ${order.status}</p>
      <small>Total: £${(order.amount_charged / 100).toFixed(2)}</small>
    `;
        orders_list.appendChild(order_item);
    });
}

async function orders_page_logout() {
    // not sure how TS namespacing works
    try {
        const res = await fetch_csrf("/api/auth", { method: "DELETE" });
        if (!res.ok) console.error("Failed to logout");
        else window.location.replace("/");
    } catch (err) {
        console.error(err);
    }
}
