const { invoke } = window.__TAURI__.core;
import { translations } from './locales.js';

// --- DOM Cache ---
let el = {};

// --- State ---
let state = {
    accounts: [],
    filterText: "",
    isEditing: false,
    theme: localStorage.getItem("theme") || "dark",
    lang: localStorage.getItem("lang") || "vi" // Default to Vietnamese if not set, or English
};

// --- Initialization ---
function init() {
    console.log("Initializing App...");

    // Cache DOM Elements
    el = {
        accountList: document.getElementById("account-list"),
        searchInput: document.getElementById("search-input"),

        // Modals
        accountModal: document.getElementById("account-modal"),
        settingsModal: document.getElementById("settings-modal"),

        // Buttons
        addAccountBtn: document.getElementById("add-account-btn"),
        settingsBtn: document.getElementById("settings-btn"),

        // Account Form
        accountForm: document.getElementById("account-form"),
        modalTitle: document.getElementById("modal-title"),
        nameInput: document.getElementById("account-name"),
        proxyInput: document.getElementById("account-proxy"),
        idInput: document.getElementById("account-id"),
        saveBtn: document.getElementById("save-account-btn"),
        deleteBtn: document.getElementById("delete-account-btn"),

        // Settings Form
        themeSelect: document.getElementById("theme-select"),
        langSelect: document.getElementById("language-select"),

        // Toasts
        toastContainer: document.getElementById("toast-container"),
    };

    // Check critical elements
    if (!el.addAccountBtn || !el.settingsBtn) {
        console.error("Critical elements missing from DOM!");
        return;
    }

    setupTheme();
    setupLanguage();
    setupEventListeners();

    // Initial Load
    updateTexts(); // Translate UI immediately
    loadAccounts();
}

// --- Localization ---
function getTrans(key) {
    const keys = key.split('.');
    let val = translations[state.lang];
    for (const k of keys) {
        val = val ? val[k] : undefined;
    }
    return val || key;
}

function updateTexts() {
    // Translate textContent
    document.querySelectorAll('[data-i18n]').forEach(elem => {
        const key = elem.getAttribute('data-i18n');
        elem.textContent = getTrans(key);
    });

    // Translate placeholders
    document.querySelectorAll('[data-i18n-placeholder]').forEach(elem => {
        const key = elem.getAttribute('data-i18n-placeholder');
        elem.placeholder = getTrans(key);
    });

    // Update dynamic button texts based on state
    if (el.saveBtn) {
        el.saveBtn.textContent = state.isEditing ? getTrans("modal.save_changes") : getTrans("modal.create");
    }
}

function setupLanguage() {
    el.langSelect.value = state.lang;
    el.langSelect.onchange = (e) => {
        state.lang = e.target.value;
        localStorage.setItem("lang", state.lang);
        updateTexts();
        renderAccounts(); // Re-render to update dynamic texts in list
        showToast("Language changed / Đã đổi ngôn ngữ", "success");
    };
}

// --- Theme Management ---
function setupTheme() {
    document.documentElement.setAttribute('data-theme', state.theme);
    el.themeSelect.value = state.theme;

    el.themeSelect.onchange = (e) => {
        state.theme = e.target.value;
        localStorage.setItem("theme", state.theme);

        if (state.theme === 'system') {
            const isDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
            document.documentElement.setAttribute('data-theme', isDark ? 'dark' : 'light');
        } else {
            document.documentElement.setAttribute('data-theme', state.theme);
        }
    };
}

// --- Data Loading & Filtering ---
async function loadAccounts() {
    const t = getTrans;
    try {
        // showToast(t("toast.loading_accounts"), "info"); // Optional: Too noisy
        state.accounts = await invoke("get_accounts");
        renderAccounts();
    } catch (e) {
        console.error("Failed to load accounts:", e);
        showToast(t("toast.load_error") + e, "error");
    }
}

function renderAccounts() {
    const t = getTrans;
    el.accountList.innerHTML = "";

    const filtered = state.accounts.filter(acc =>
        acc.name.toLowerCase().includes(state.filterText.toLowerCase())
    );

    if (filtered.length === 0) {
        el.accountList.innerHTML = `
            <div style="text-align: center; color: var(--text-muted); padding: 2rem;">
                <p>${state.filterText ? t("app.no_matches") : t("app.no_accounts")}</p>
                ${!state.filterText ? `<small>${t("app.click_add")}</small>` : ''}
            </div>
        `;
        return;
    }

    filtered.forEach((acc) => {
        const card = document.createElement("div");
        card.className = `account-card ${acc.is_open ? 'active' : ''}`;

        card.innerHTML = `
            <div class="account-info">
                <div class="account-name" title="${acc.name}">${acc.name}</div>
                <div class="account-status ${acc.is_open ? 'online' : ''}" title="${acc.is_open ? t("app.running") : 'Offline'}"></div>
            </div>
            ${acc.proxy ? `
                <div class="proxy-badge" title="${acc.proxy}">
                    <span>${t("app.proxy_badge")}</span>
                </div>
            ` : ''}
            <div class="account-actions">
                <button class="action-btn open-btn" data-id="${acc.id}">
                    ${acc.is_open ? t("app.switch_to") : t("app.open")}
                </button>
                ${acc.is_open ?
                `<button class="action-btn close-btn" data-id="${acc.id}">${t("app.close")}</button>` :
                ''
            }
                <button class="action-btn edit-btn" data-id="${acc.id}">${t("app.settings")}</button>
            </div>
        `;

        // Event Handling
        card.onclick = (e) => {
            if (e.target === card || e.target.classList.contains('account-info') || e.target.classList.contains('account-name')) {
                openAccount(acc.id);
            }
        };

        const openBtn = card.querySelector(".open-btn");
        if (openBtn) openBtn.onclick = (e) => { e.stopPropagation(); openAccount(acc.id); };

        const closeBtn = card.querySelector(".close-btn");
        if (closeBtn) closeBtn.onclick = (e) => { e.stopPropagation(); closeAccount(acc.id); };

        const editBtn = card.querySelector(".edit-btn");
        if (editBtn) editBtn.onclick = (e) => { e.stopPropagation(); openEditModal(acc); };

        el.accountList.appendChild(card);
    });
}

// --- Actions ---
async function openAccount(id) {
    const t = getTrans;
    try {
        await invoke("open_account", { id });
        await loadAccounts();
        showToast(t("toast.launching"), "info");
    } catch (e) {
        console.error("Failed to open:", e);
        showToast(t("toast.launch_error") + e, "error");
    }
}

async function closeAccount(id) {
    const t = getTrans;
    try {
        await invoke("close_account", { id });
        await loadAccounts();
        showToast(t("toast.close_success"), "success");
    } catch (e) {
        showToast(t("toast.close_error") + e, "error");
    }
}

async function handleSave(e) {
    e.preventDefault();
    const t = getTrans;

    const name = el.nameInput.value.trim();
    const proxy = el.proxyInput.value.trim() || null;
    const id = el.idInput.value;

    if (!name) {
        showToast(t("toast.name_required"), "error");
        return;
    }

    // Set loading state
    const originalBtnText = el.saveBtn.textContent;
    el.saveBtn.textContent = t("modal.saving");
    el.saveBtn.disabled = true;

    try {
        if (state.isEditing && id) {
            await invoke("rename_account", { id, newName: name });
            await invoke("update_account_proxy", { id, proxy });
            showToast(t("toast.save_success"), "success");
        } else {
            await invoke("add_account", { name, proxy });
            showToast(t("toast.create_success"), "success");
        }

        closeModal(el.accountModal);
        loadAccounts();
    } catch (e) {
        console.error("Save Error:", e);
        showToast(t("toast.save_error") + e, "error");
    } finally {
        // Reset loading state
        el.saveBtn.textContent = originalBtnText; // Will be updated by updateTexts anyway based on isEditing
        el.saveBtn.disabled = false;
        // Re-run text update to ensure correct label (Create vs Save)
        updateTexts();
    }
}

async function handleDelete() {
    const t = getTrans;
    const id = el.idInput.value;
    if (!id) return;

    if (confirm(t("modal.confirm_delete"))) {
        try {
            await invoke("remove_account", { id });
            closeModal(el.accountModal);
            loadAccounts();
            showToast(t("toast.delete_success"), "success");
        } catch (e) {
            showToast(t("toast.delete_error") + e, "error");
        }
    }
}

// --- UI Logic ---
function openAddModal() {
    const t = getTrans;
    state.isEditing = false;

    el.modalTitle.textContent = t("modal.add_title");
    el.accountForm.reset();
    el.idInput.value = "";
    el.deleteBtn.classList.add("hidden");
    el.saveBtn.textContent = t("modal.create");

    openModal(el.accountModal);
    el.nameInput.focus();
}

function openEditModal(account) {
    const t = getTrans;
    state.isEditing = true;

    el.modalTitle.textContent = t("modal.edit_title");
    el.idInput.value = account.id;
    el.nameInput.value = account.name;
    el.proxyInput.value = account.proxy || "";
    el.deleteBtn.classList.remove("hidden");
    el.saveBtn.textContent = t("modal.save_changes");

    openModal(el.accountModal);
}

function openModal(modal) {
    modal.classList.remove("hidden");
}

function closeModal(modal) {
    modal.classList.add("hidden");
    if (modal === el.accountModal) {
        el.accountForm.reset();
        state.isEditing = false; // Reset state
    }
}

function showToast(message, type = "info") {
    const toast = document.createElement("div");
    toast.className = `toast ${type}`;

    let icon = "ℹ️";
    if (type === "success") icon = "✅";
    if (type === "error") icon = "⚠️";

    toast.innerHTML = `<span class="toast-icon">${icon}</span> ${message}`;

    el.toastContainer.appendChild(toast);

    setTimeout(() => {
        toast.style.opacity = "0";
        toast.style.transform = "translateX(100%)";
        setTimeout(() => toast.remove(), 300);
    }, 3000);
}

// --- Event Listeners ---
function setupEventListeners() {
    console.log("Setting up event listeners...");

    if (el.addAccountBtn) {
        el.addAccountBtn.onclick = () => {
            console.log("Add Account Clicked");
            openAddModal();
        };
    } else console.warn("Add Account Btn missing");

    if (el.settingsBtn) {
        el.settingsBtn.onclick = () => {
            console.log("Settings Clicked");
            openModal(el.settingsModal);
        };
    }

    // Search
    if (el.searchInput) {
        el.searchInput.oninput = (e) => {
            state.filterText = e.target.value;
            renderAccounts();
        };
    }

    // Modals Close
    document.querySelectorAll(".close-modal-btn, .cancel-btn").forEach(btn => {
        btn.onclick = (e) => {
            const modal = e.target.closest(".modal");
            closeModal(modal);
        };
    });

    window.onclick = (e) => {
        if (e.target.classList.contains("modal")) {
            closeModal(e.target);
        }
    };

    // Forms
    if (el.accountForm) el.accountForm.onsubmit = handleSave;
    if (el.deleteBtn) el.deleteBtn.onclick = handleDelete;

    // Auto-refresh status
    setInterval(loadAccounts, 5000);
}

// Start when DOM is ready
window.addEventListener('DOMContentLoaded', init);
