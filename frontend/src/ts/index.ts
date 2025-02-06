async function checkAuthRedirect() {
    let response = await fetch("/api/auth/whoami");
    let redirect_message = document.getElementById("redirect-message");
    if (redirect_message === null) {
        throw new Error("redirect-message element not defined");
    }
    let redirect_link = document.getElementById("redirect-link");
    if (redirect_link === null || !(redirect_link instanceof HTMLAnchorElement)) {
        throw new Error("redirect-link is not a link.");
    }
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
