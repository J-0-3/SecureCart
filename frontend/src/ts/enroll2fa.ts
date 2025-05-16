let mfa_secret: string = "";

function show_enroll2fa_failure_modal(message: string) {
  const modalBody = document.getElementById("failure-modal-body");
  if (modalBody) {
    modalBody.textContent = message;
  }
  const modalElement = document.getElementById("failure-modal");
  if (modalElement) {
    const modal = new bootstrap.Modal(modalElement);
    modal.show();
  }
}

async function fetch_2fa_data() {
  const response = await fetch_csrf("/api/users/self/2fa/new");
  if (!response.ok) {
    show_enroll2fa_failure_modal("Failed to load 2FA data.");
    return;
  }
  const data = await response.json();
  mfa_secret = data.secret;
  const qr = document.getElementById("qr-code")! as HTMLImageElement;
  qr.src = "data:image/png;base64," + data.qr;
}

async function attempt_2fa_enrollment() {
  const codeInput = document.getElementById("code") as HTMLInputElement;
  const code = codeInput.value.trim();
  if (!code) {
    show_enroll2fa_failure_modal(
      "Please enter the code from your authenticator app.",
    );
    return;
  }
  const response = await fetch_csrf("/api/users/self/2fa", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      secret: mfa_secret,
      code: code,
    }),
  });
  if (response.ok) {
    window.location.replace("/user.html");
  } else if (response.status === 403) {
    show_enroll2fa_failure_modal("2fa code is incorrect. Please try again");
  } else {
    show_enroll2fa_failure_modal("Failed to enroll 2FA. Please try again.");
  }
}

document.addEventListener("DOMContentLoaded", () => {
  fetch_2fa_data();
  const form = document.getElementById("enroll2fa-form");
  if (form) {
    form.addEventListener("submit", (evt) => {
      evt.preventDefault();
      attempt_2fa_enrollment();
      return false;
    });
  }
});
