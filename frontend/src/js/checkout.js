document.addEventListener("DOMContentLoaded", init_checkout);

function init_checkout() {
  const cart = JSON.parse(localStorage.getItem("cart") || "[]");
  if (cart.length === 0) {
    window.location.replace("/store.html");
    return;
  }
  const product_counts = {};
  for (const id of cart) {
    product_counts[id] = (product_counts[id] || 0) + 1;
  }

  render_order_summary(product_counts);

  document.getElementById("checkout_button").addEventListener("click", () => {
    submit_order(product_counts);
  });
  document.getElementById("logout").addEventListener("click", () => {
    logout();
  });
}

async function fetch_product(product_id) {
  const resp = await fetch_csrf(`/api/products/${product_id}`);
  if (!resp.ok) throw new Error(`Failed to fetch product ${product_id}`);
  return await resp.json();
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

async function render_order_summary(product_counts) {
  const order_items = document.getElementById("order_items");
  order_items.innerHTML = "";
  let total_price = 0;

  for (const id of Object.keys(product_counts)) {
    try {
      const product = await fetch_product(id);
      const count = product_counts[id];
      total_price += product.price * count;

      const li = document.createElement("li");
      li.className =
        "list-group-item d-flex justify-content-between align-items-center";
      li.innerHTML = `${product.name} - £${(product.price / 100).toFixed(2)} x ${count}`;

      const subtract_btn = document.createElement("button");
      subtract_btn.className = "btn btn-danger btn-sm";
      subtract_btn.textContent = "-";
      subtract_btn.onclick = () => remove_item(id, product_counts);

      li.appendChild(subtract_btn);
      order_items.appendChild(li);
    } catch (err) {
      console.error("Failed to fetch product", id, err);
    }
  }
  update_total_price(total_price);

  if (Object.keys(product_counts).length === 0) {
    window.location.replace("/store.html");
  }
}

function update_total_price(total) {
  let total_price_element = document.getElementById("total_price");
  if (!total_price_element) {
    total_price_element = document.createElement("div");
    total_price_element.id = "total_price";
    total_price_element.className = "fw-bold fs-4 mt-3 text-end";
    document.getElementById("order_summary").appendChild(total_price_element);
  }
  total_price_element.textContent = `Total: £${(total / 100).toFixed(2)}`;
}

function remove_item(product_id, product_counts) {
  if (product_counts[product_id] > 1) {
    product_counts[product_id]--;
  } else {
    delete product_counts[product_id];
  }

  let updated_cart = [];
  for (const id of Object.keys(product_counts)) {
    for (let i = 0; i < product_counts[id]; i++) {
      updated_cart.push(id);
    }
  }
  localStorage.setItem("cart", JSON.stringify(updated_cart));

  render_order_summary(product_counts);
}

async function submit_order(product_counts) {
  const checkout_button = document.getElementById("checkout_button");
  checkout_button.disabled = true;
  const order_payload = { products: [] };
  for (const product of Object.keys(product_counts)) {
    order_payload.products.push({
      product: product,
      count: product_counts[product],
    });
  }
  let new_order_res = await fetch_csrf("/api/orders", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(order_payload),
  });
  if (!new_order_res.ok) {
    console.error("Failed to create order", new_order_res.status);
    return;
  }
  let new_order_json = await new_order_res.json();
  let checkout_res = await fetch_csrf("/api/checkout", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ order_id: new_order_json.id }),
  });
  if (!checkout_res.ok) {
    console.error("Failed to initiate checkout", checkout_res.status);
    return;
  }
  let checkout_json = await checkout_res.json();
  if (!checkout_json.payment_required) {
    localStorage.removeItem("cart");
    window.location.replace(`/order.html?order=${new_order_json.id}`);
    return;
  }
  setup_stripe_payment(
    checkout_json.payment_info.publishable_key,
    checkout_json.payment_info.client_secret,
    new_order_json.id,
  );
}

function setup_stripe_payment(publishable_key, client_secret, order_id) {
  document
    .querySelectorAll("#order_items .btn-danger.btn-sm")
    .forEach((btn) => btn.remove());
  document.getElementById("payment_form_container").hidden = false;
  const stripe = Stripe(publishable_key);
  const elements = stripe.elements({ clientSecret: client_secret });
  const payment_element = elements.create("payment");
  payment_element.mount("#payment_element");
  document
    .getElementById("payment_form")
    .addEventListener("submit", async function (event) {
      event.preventDefault();
      const submit_payment = document.getElementById("submit_payment");
      submit_payment.disabled = true;
      const result = await stripe.confirmPayment({
        elements: elements,
        confirmParams: {
          return_url: `${window.location.origin}/order.html?order=${order_id}&message=Your+order+has+been+placed+successfully`,
        },
        redirect: "if_required",
      });
      if (result.error) {
        document.getElementById("payment_errors").textContent =
          result.error.message || "Payment failed";
        submit_payment.disabled = false;
        return;
      }
      localStorage.removeItem("cart");
      window.location.replace(`/order.html?order=${order_id}`);
    });
}
