interface AuthenticationResponse {
    mfa_required: boolean,
    is_admin?: boolean
}

function show_failure_modal(message: string) {
    document.getElementById("failure-modal-body")!.textContent = message;
    const modal = new bootstrap.Modal(document.getElementById("failure-modal")!);
    modal.show()
}
async function attempt_login() {
    const email = (document.getElementById("email")! as HTMLInputElement).value;
    const password = (document.getElementById("password")! as HTMLInputElement).value;
    const response = await fetch("/api/auth", {
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
    if (response.status === 200) {
        const body: AuthenticationResponse = await response.json();
        if (body.mfa_required) {
            alert("Todo: Add a 2FA frontend here (for testing POST to /api/auth/2fa)");
        } else {
            window.location.replace("/");
        }
    } else if (response.status === 401) {
        show_failure_modal("Incorrect email or password. Please try again.")
    } else if (response.status === 500) {
        show_failure_modal("Oops. Looks like something went wrong on our end. Please try that again.")
    } else if (response.status === 422) {
        show_failure_modal("Your request was malformed and unable to be processed.")
    }
}

function register_event_handlers() {
    document.getElementById("login-form")!.addEventListener("submit", (evt) => {
        evt.preventDefault();
        attempt_login();
        return false // this seems to stop the form reloading with preventDefault
    });
}

document.addEventListener('DOMContentLoaded', register_event_handlers);
