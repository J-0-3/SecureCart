async function check_auth_and_redirect() {
    const response = await fetch("/api/auth/whoami");
    const redirect_message = document.getElementById("redirect-message")!;
    const redirect_link = <HTMLAnchorElement>document.getElementById("redirect-link")!;
    if (response.status == 200) {
        redirect_link.href = "/store.html";
        redirect_message.hidden = false;
        window.location.replace("/store.html");
    } else if (response.status == 401) {
        redirect_link.href = "/login.html";
        redirect_message.hidden = false;
        window.location.replace("/login.html");
    } else {
        redirect_link.href = `/error${response.status}.html`;
        redirect_message.hidden = false;
        window.location.replace(`/error/${response.status}.html`)
    }
}

document.addEventListener("DOMContentLoaded", check_auth_and_redirect)
