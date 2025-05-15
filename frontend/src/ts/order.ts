interface Order {
  id: string;
  amount_charged: number;
  order_placed: string;
  user_id: string;
  status: string;
}

interface OrderResponse {
  order: Order;
  items: [string, number][];
}

interface User {
  email: string;
  forename: string;
  surname: string;
  address: string;
}

document.addEventListener("DOMContentLoaded", init_order_page);

async function init_order_page() {
  const url_params = new URLSearchParams(window.location.search);
  const order_id = url_params.get("order");
  const message = url_params.get("message");

  if (order_id === null) {
    window.location.replace("/index.html");
    return;
  }

  if (message) {
    show_message_modal(message);
  }

  const auth_res = await fetch("/api/auth/check");
  if (!auth_res.ok) {
    window.location.replace("/index.html");
  }

  const is_admin = (await fetch("/api/auth/check/admin")).ok;

  let order_data: OrderResponse;
  try {
    const order_res = await fetch_csrf(`/api/orders/${order_id}`);
    if (!order_res.ok) {
      if (order_res.status === 401 || order_res.status === 403) {
        window.location.replace("/index.html");
        return;
      } else if (order_res.status === 404) {
        window.location.replace("/error/404.html");
      }
      throw new Error("Failed to fetch order data");
    }
    order_data = await order_res.json();
  } catch (err) {
    console.error("Error fetching order:", err);
    window.location.replace("/index.html");
    return;
  }

  await render_order(order_data, is_admin);

  if (is_admin && order_data.order.status === "Confirmed") {
    show_fulfil_button(order_id);
  }
}

function show_message_modal(message: string): void {
  document.getElementById("alert-modal-body")!.textContent = message;
  const modal = new bootstrap.Modal(document.getElementById("alert-modal")!);
  modal.show();
}

async function render_order(
  order_data: OrderResponse,
  is_admin: boolean,
): Promise<void> {
  const container = document.getElementById("order_container");
  if (!container) return;

  let user_info: User | null = null;
  try {
    const user_res = is_admin
      ? await fetch_csrf(`/api/users/${order_data.order.user_id}`)
      : await fetch_csrf("/api/users/self");
    if (user_res.ok) {
      user_info = await user_res.json();
    }
  } catch (err) {
    console.error("Error fetching user info:", err);
  }

  container.innerHTML = `
    <h2>Order #${order_data.order.id}</h2>
    <p><strong>Status:</strong> ${order_data.order.status}</p>
    <p><strong>Order Placed:</strong> ${new Date(order_data.order.order_placed).toLocaleString()}</p>
    <p><strong>Amount Charged:</strong> £${(order_data.order.amount_charged / 100).toFixed(2)}</p>
    <p><strong>User Email:</strong> <span id="order-user-email"></span></p>
    <p><strong>Shipping Address:</strong> <span id="order-user-address"></span></p>
    <h3>Items</h3>
    <ul id="order_items_list" class="list-group"></ul>
  `;

  document.getElementById("order-user-email")!.textContent = user_info
    ? user_info.email
    : order_data.order.user_id.toString();

  document.getElementById("order-user-address")!.textContent = user_info
    ? user_info.address
    : "";

  const items_list = document.getElementById("order_items_list");
  if (!items_list) return;
  for (const item of order_data.items) {
    const product_url = item[0];
    const quantity = item[1];
    await fetch_product_and_append(product_url, quantity, items_list);
  }
}

async function fetch_product_and_append(
  uri: string,
  quantity: number,
  container: HTMLElement,
): Promise<void> {
  try {
    const resp = await fetch_csrf(uri);
    if (!resp.ok) throw new Error("Failed to fetch product");
    const product = await resp.json();
    const li = document.createElement("li");
    li.className =
      "list-group-item d-flex justify-content-between align-items-center";
    li.textContent = `${product.name} - £${(product.price / 100).toFixed(2)} x ${quantity}`;
    container.appendChild(li);
  } catch (err) {
    console.error("Error fetching product:", uri, err);
    const li = document.createElement("li");
    li.className = "list-group-item";
    li.textContent = "Failed to load";
    container.appendChild(li);
  }
}

function show_fulfil_button(order_id: string): void {
  const fulfil_container = document.getElementById("fulfil_button_container")!;
  const fulfil_button = document.createElement("button");
  fulfil_button.id = "fulfil_button";
  fulfil_button.className = "btn btn-primary";
  fulfil_button.textContent = "Fulfil Order";
  fulfil_button.addEventListener("click", async () => {
    try {
      const res = await fetch_csrf(`/api/orders/${order_id}/fulfil`, {
        method: "POST",
      });
      if (!res.ok) throw new Error("Failed to fulfil order");
      const order_res = await fetch_csrf(`/api/orders/${order_id}`);
      if (!order_res.ok) throw new Error("Failed to fetch updated order");
      const order_data: OrderResponse = await order_res.json();
      await render_order(order_data, true);
      if (order_data.order.status === "Fulfilled") {
        fulfil_container.innerHTML = "";
        show_message_modal("Order fulfilled successfully");
      }
    } catch (err) {
      console.error(err);
      show_message_modal(`Error fulfilling order: ${err}`);
    }
  });
  fulfil_container.appendChild(fulfil_button);
}
