const { invoke } = window.__TAURI__.core;

const accountListEl = document.getElementById("account-list");
const addBtn = document.getElementById("add-account-btn");

async function loadAccounts() {
    try {
        const accounts = await invoke("get_accounts");
        accountListEl.innerHTML = "";
        accounts.forEach((acc) => {
            const div = document.createElement("div");
            div.className = "account-item";
            div.style.marginBottom = "10px";
            div.style.padding = "10px";
            div.style.background = "#333";
            div.style.borderRadius = "4px";
            div.style.display = "flex";
            div.style.justifyContent = "space-between";
            div.style.alignItems = "center";

            div.innerHTML = `
        <span>${acc.name} ${acc.proxy ? '(Proxy)' : ''}</span>
        <div>
          <button onclick="window.openAccount('${acc.id}')" style="margin-right: 5px;">Open</button>
          <button onclick="window.removeAccount('${acc.id}')">Remove</button>
        </div>
      `;
            accountListEl.appendChild(div);
        });
    } catch (e) {
        console.error(e);
        alert("Error loading accounts: " + e);
    }
}

window.openAccount = async (id) => {
    try {
        await invoke("open_account", { id });
    } catch (e) {
        alert("Error opening account: " + e);
    }
};

window.removeAccount = async (id) => {
    if (confirm("Delete this account?")) {
        try {
            await invoke("remove_account", { id });
            loadAccounts();
        } catch (e) {
            alert("Error removing account: " + e);
        }
    }
};

addBtn.onclick = async () => {
    const name = prompt("Enter Account Name:");
    if (name) {
        const proxy = prompt("Enter Proxy (optional, e.g. http://user:pass@host:port):");
        try {
            await invoke("add_account", { name, proxy: proxy || null });
            loadAccounts();
        } catch (e) {
            alert("Error adding account: " + e);
        }
    }
};

loadAccounts();
