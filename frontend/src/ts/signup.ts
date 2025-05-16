function show_signup_failure_modal(message: string) {
  const modal_body = document.getElementById("failure-modal-body");
  if (modal_body) {
    modal_body.textContent = message;
  }
  const modal_element = document.getElementById("failure-modal");
  if (modal_element) {
    const modal = new bootstrap.Modal(modal_element);
    modal.show();
  }
}

class RegistrationErrorResponse {
  message?: string;
}
async function attempt_registration() {
  const forename = (
    document.getElementById("forename") as HTMLInputElement
  ).value.trim();
  if (forename.length === 0) {
    show_signup_failure_modal("Forename cannot be empty");
    return;
  }
  const surname = (
    document.getElementById("surname") as HTMLInputElement
  ).value.trim();
  if (surname.length === 0) {
    show_signup_failure_modal("Surname cannot be empty");
    return;
  }
  const email = (
    document.getElementById("email") as HTMLInputElement
  ).value.trim();
  if (email.length === 0) {
    show_signup_failure_modal("email cannot be empty");
    return;
  }
  const address = (
    document.getElementById("address") as HTMLInputElement
  ).value.trim();
  if (address.length === 0) {
    show_signup_failure_modal("Address cannot be empty");
    return;
  }
  const password = (document.getElementById("password") as HTMLInputElement)
    .value;
  const confirm = (
    document.getElementById("confirm-password") as HTMLInputElement
  ).value;

  if (password !== confirm) {
    show_signup_failure_modal("Passwords do not match. Please try again.");
    return;
  }

  const signup_response = await fetch("/api/registration", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      user_data: {
        forename: forename,
        surname: surname,
        email: email,
        address: address,
      },
    }),
  });

  if (!signup_response.ok) {
    let error_msg = "Registration failed during signup. Please try again.";
    if (signup_response.status === 422) {
      const body: RegistrationErrorResponse = await signup_response.json();
      error_msg = `Your signup request was malformed: ${body.message}`;
    } else if (signup_response.status === 500) {
      error_msg = "Server error during signup.";
    } else if (signup_response.status === 409) {
      error_msg = "Email address is already in use.";
    }
    show_signup_failure_modal(error_msg);
    return;
  }

  const credential_response = await fetch_csrf("/api/registration/credential", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      credential: {
        Password: {
          password: password,
        },
      },
    }),
  });

  if (credential_response.ok) {
    window.location.replace("/login.html");
  } else {
    let error_msg =
      "Registration failed during credential setup. Please try again.";
    if (credential_response.status === 422) {
      const body: RegistrationErrorResponse = await credential_response.json();
      error_msg = `Your credential request was malformed: ${body.message}`;
    } else if (credential_response.status === 500) {
      error_msg = "Server error during credential setup.";
    }
    show_signup_failure_modal(error_msg);
  }
}

document.addEventListener("DOMContentLoaded", () => {
  const form = document.getElementById("registration-form");
  if (form) {
    form.addEventListener("submit", (evt) => {
      evt.preventDefault();
      attempt_registration();
      return false;
    });
  }
});
