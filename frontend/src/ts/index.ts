async function check_customer(): Promise<number> {
    const response = await fetch("/api/auth/check/customer");
    return response.status
}

async function check_admin(): Promise<number> {
    const response = await fetch("/api/auth/check/admin");
    return response.status
}

async function check_auth_and_redirect() {
    const redirect_message = document.getElementById("redirect-message")!;
    const redirect_link = <HTMLAnchorElement>document.getElementById("redirect-link")!;
    const admin_response = await check_admin();
    switch (admin_response) {
        case 200:
            redirect_link.href = "/admin/dashboard.html";
            redirect_message.hidden = false;
            window.location.replace("/admin/dashboard.html");
            return;
        case 401:
            break;
        default:
            redirect_link.href = `/error/${admin_response}.html`;
            redirect_message.hidden = false;
            window.location.replace(`/error/${admin_response}.html`);
            return;
    }
    const customer_response = await check_customer();
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

document.addEventListener("DOMContentLoaded", check_auth_and_redirect)
