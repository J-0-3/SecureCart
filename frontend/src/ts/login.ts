async function attempt_login() {
    let email = (<HTMLInputElement>document.getElementById("email")).value;
    let password = (<HTMLInputElement>document.getElementById("password")).value;
    let response = await fetch("/api/auth/login", {
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
    switch (response.status) {
        case 401:
            let failure_modal = new bootstrap.Modal(document.getElementById('loginFailedModal')!, {});
            failure_modal.show();
            break;
        case 200:
            let body = await response.json();
            if (body.mfa_required) {
                alert("Redirect to MFA");
            } else {
                window.location.replace("/");
            }
            break
    }
}
