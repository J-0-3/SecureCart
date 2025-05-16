function show_2fa_failure_modal(message: string) {
  document.getElementById("failure-modal-body")!.textContent = message;
  const modal = new bootstrap.Modal(document.getElementById("failure-modal")!);
  modal.show();
}
async function attempt_2fa() {
  const code = (document.getElementById("totp-code")! as HTMLInputElement)
    .value;
  const response = await fetch_csrf("/api/auth/2fa", {
    method: "post",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      credential: {
        Totp: {
          code: code,
        },
      },
    }),
  });
  if (response.status === 200) {
    window.location.replace("/");
  } else if (response.status === 401) {
    show_2fa_failure_modal("Incorrect 2fa code. Please try again.");
  } else if (response.status === 500) {
    show_2fa_failure_modal(
      "Oops. Looks like something went wrong on our end. Please try that again.",
    );
  } else if (response.status === 422) {
    show_2fa_failure_modal(
      "Your request was malformed and unable to be processed.",
    );
  }
}

document.addEventListener("DOMContentLoaded", () => {
  document.getElementById("login-form")!.addEventListener("submit", (evt) => {
    evt.preventDefault();
    attempt_2fa();
    return false; // this seems to stop the form reloading with preventDefault
  });
});
