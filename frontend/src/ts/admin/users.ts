interface User {
    id: string;
    email: string;
    forename: string;
    surname: string;
    address: string;
    role: string;
}

interface UserResponse {
    users: User[];
}

document.addEventListener("DOMContentLoaded", async () => {
    if (!(await fetch_csrf("/api/auth/check/admin")).ok) {
        window.location.replace("/index.html");
        return;
    }
    document
        .getElementById("logout")!
        .addEventListener("click", users_page_logout);
    document
        .getElementById("search_users")!
        .addEventListener("click", load_and_render_users);
    load_and_render_users();
});

async function load_and_render_users() {
    const role = (document.getElementById("role_filter") as HTMLSelectElement)
        .value;
    const email = (document.getElementById("email_search") as HTMLInputElement)
        .value;
    let url = "/api/users";

    const params: string[] = [];
    if (role) {
        params.push(`role=${encodeURIComponent(role)}`);
    }
    if (email) {
        params.push(`email=${encodeURIComponent(email)}`);
    }
    if (params.length > 0) {
        url += `?${params.join("&")}`;
    }

    try {
        const res = await fetch_csrf(url);
        if (!res.ok) throw new Error("Failed to fetch users");
        const user_response: UserResponse = await res.json();
        const users = user_response.users;
        render_users(users);
    } catch (err) {
        console.error(err);
    }
}

function render_users(users: User[]) {
    const users_list = document.getElementById("users_list")!;
    users_list.innerHTML = users.length
        ? ""
        : `<div class="alert alert-info">No users found with current criteria.</div>`;
    users.forEach((user) => {
        const user_item = document.createElement("a");
        user_item.href = `/user.html?user_id=${user.id}`;
        user_item.className = "list-group-item list-group-item-action";
        user_item.innerHTML = `
      <div class="d-flex w-100 justify-content-between">
        <h5 class="mb-1" id="user-${user.id}-name"></h5>
        <small><span id="user-${user.id}-email"></span></small>
      </div>
      <p class="mb-1">Role: ${user.role}</p>
      <small>Address: <span id="user-${user.id}-address"></span></small>
    `;
        users_list.appendChild(user_item);
        document.getElementById(`user-${user.id}-name`)!.textContent =
            `${user.forename} ${user.surname}`;
        document.getElementById(`user-${user.id}-email`)!.textContent = user.email;
        document.getElementById(`user-${user.id}-address`)!.textContent =
            user.address;
    });
}

async function users_page_logout() {
    try {
        const res = await fetch_csrf("/api/auth", { method: "DELETE" });
        if (!res.ok) console.error("Failed to logout");
        else window.location.replace("/");
    } catch (err) {
        console.error(err);
    }
}
