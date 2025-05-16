async function check_customer(): Promise<number> {
  const response = await fetch_csrf("/api/auth/check/customer");
  return response.status;
}

async function check_admin(): Promise<number> {
  const response = await fetch_csrf("/api/auth/check/admin");
  return response.status;
}

async function check_auth_and_redirect() {
  const redirect_message = document.getElementById("redirect-message")!;
  const redirect_link = <HTMLAnchorElement>(
    document.getElementById("redirect-link")!
  );
  let admin_response;
  try {
    admin_response = await check_admin();
  } catch (_) {
    redirect_link.href = "/login.html";
    redirect_link.hidden = false;
    window.location.replace("/login.html");
    return;
  }
  switch (admin_response) {
    case 200:
      redirect_link.href = "/admin/orders.html";
      redirect_message.hidden = false;
      window.location.replace("/admin/orders.html");
      return;
    case 401:
      break;
    default:
      redirect_link.href = `/error/${admin_response}.html`;
      redirect_message.hidden = false;
      window.location.replace(`/error/${admin_response}.html`);
      return;
  }
  let customer_response;
  try {
    customer_response = await check_customer();
  } catch (_) {
    redirect_link.href = "/login.html";
    redirect_message.hidden = false;
    window.location.replace("/login.html");
  }
  switch (customer_response) {
    case 200:
      redirect_link.href = "/store.html";
      redirect_message.hidden = false;
      window.location.replace("/store.html");
      break;
    case 401:
      redirect_link.href = "/login.html";
      redirect_message.hidden = false;
      window.location.replace("/login.html");
      break;
    default:
      redirect_link.href = `/error/${customer_response}.html`;
      redirect_message.hidden = false;
      window.location.replace(`/error/${customer_response}.html`);
  }
}

document.addEventListener("DOMContentLoaded", check_auth_and_redirect);
