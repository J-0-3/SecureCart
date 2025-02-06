async function attempt_login() {
    const email = (<HTMLInputElement>document.getElementById("email")).value;
    const password = (<HTMLInputElement>document.getElementById("password")).value;
    const failure_modal = new bootstrap.Modal(document.getElementById('loginFailedModal')!, {});
    const response = await fetch("/api/auth/login", {
        method: "post",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
            email: email,
            credential: {
                Password: {
                    password: password
                }
            }
        })
    });
    const body = await response.json();
    switch (response.status) {
        case 401:
            failure_modal.show();
            break;
        case 200:
            if (body.mfa_required) {
                alert("Redirect to MFA");
            } else {
                window.location.replace("/");
            }
            break
    }
}

function register_event_handlers() {
    document.getElementById("login-form")!.addEventListener("submit", () => { attempt_login(); return false });
}

document.addEventListener('DOMContentLoaded', register_event_handlers);
