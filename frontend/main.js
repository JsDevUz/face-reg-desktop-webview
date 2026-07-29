import axios, { isAxiosError } from "axios";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { fetch as tauriFetch } from "@tauri-apps/plugin-http";

const API_BASE_URL = "https://api.tayin.uz";
const client = axios.create({
  baseURL: API_BASE_URL,
  headers: {
    Accept: "application/json",
    "Content-Type": "application/json; charset=utf-8",
  },
});

const form = document.getElementById("login-form");
const phoneInput = document.getElementById("phone");
const passwordInput = document.getElementById("password");
const errorElement = document.getElementById("error");
const responseDetails = document.getElementById("response-details");
const responseBody = document.getElementById("response-body");
const submitButton = document.getElementById("submit");
const blockedCard = document.getElementById("blocked-card");
const blockedStore = document.getElementById("blocked-store");
const blockedMessage = document.getElementById("blocked-message");
const backToLogin = document.getElementById("back-to-login");

function localPhoneDigits(value) {
  const digits = value.replace(/\D/g, "");
  const withoutCountry = digits.startsWith("998") ? digits.slice(3) : digits;
  return withoutCountry.slice(0, 9);
}

function formatPhone(value) {
  const digits = localPhoneDigits(value);
  const parts = [
    digits.slice(0, 2),
    digits.slice(2, 5),
    digits.slice(5, 7),
    digits.slice(7, 9),
  ].filter(Boolean);
  return `+998${parts.length ? ` ${parts.join(" ")}` : ""}`;
}

function findAccessToken(payload) {
  if (!payload || typeof payload !== "object") return null;
  for (const key of ["access_token", "accessToken", "token"]) {
    if (typeof payload[key] === "string") return payload[key];
  }
  return findAccessToken(payload.data);
}

function errorMessage(payload, fallback) {
  return typeof payload?.message === "string" && payload.message
    ? payload.message
    : fallback;
}

async function login(phone, password) {
  const headers = {
    Accept: "application/json",
    "Content-Type": "application/json; charset=utf-8",
  };

  if (isTauri()) {
    const response = await tauriFetch(`${API_BASE_URL}/v1/login`, {
      method: "POST",
      headers,
      body: JSON.stringify({ phone, password }),
    });
    let payload = null;
    try {
      payload = await response.json();
    } catch {
      // Server ayrim xatolarda bo‘sh body qaytarishi mumkin.
    }
    console.info("FaceReg login response", {
      status: response.status,
      ok: response.ok,
      payload,
    });
    if (!response.ok) {
      const error = new Error(
        errorMessage(payload, `API xatosi (${response.status})`),
      );
      error.diagnostics = { status: response.status, payload };
      throw error;
    }
    return findAccessToken(payload);
  }

  try {
    const response = await client.post("/v1/login", { phone, password });
    return findAccessToken(response.data);
  } catch (reason) {
    if (isAxiosError(reason) && reason.response) {
      const error = new Error(
        errorMessage(reason.response.data, `API xatosi (${reason.response.status})`),
      );
      error.diagnostics = {
        status: reason.response.status,
        payload: reason.response.data,
      };
      throw error;
    }
    throw reason;
  }
}

function showError(message, diagnostics = null) {
  errorElement.textContent = message;
  errorElement.classList.toggle("visible", Boolean(message));
  responseDetails.classList.toggle("visible", Boolean(diagnostics));
  responseBody.textContent = diagnostics
    ? JSON.stringify(diagnostics, null, 2)
    : "";
}

function showBlocked(error) {
  form.hidden = true;
  blockedStore.textContent = error.storeName ? `Do‘kon: ${error.storeName}` : "";
  const details = [];
  if (error.terminalId) details.push(`EPOS: ${error.terminalId}`);
  if (error.allowedTerminalIds?.length) {
    details.push(`Pharma: ${error.allowedTerminalIds.join(", ")}`);
  }
  blockedMessage.textContent = [
    error.message || "Bu foydalanuvchi uchun terminal mos kelmadi.",
    ...details,
    error.diagnostics
      ? `\nEPOS diagnostika:\n${JSON.stringify(error.diagnostics, null, 2)}`
      : "",
  ].join("\n");
  blockedCard.hidden = false;
}

backToLogin.addEventListener("click", () => {
  blockedCard.hidden = true;
  form.hidden = false;
  passwordInput.value = "";
  submitButton.disabled = false;
  submitButton.textContent = "Kirish";
  passwordInput.focus();
});

phoneInput.addEventListener("input", () => {
  phoneInput.value = formatPhone(phoneInput.value);
});

phoneInput.addEventListener("focus", () => {
  phoneInput.setSelectionRange(phoneInput.value.length, phoneInput.value.length);
});

form.addEventListener("submit", async (event) => {
  event.preventDefault();
  const digits = localPhoneDigits(phoneInput.value);
  if (digits.length !== 9) {
    showError("Telefon raqamini to‘liq kiriting");
    return;
  }
  if (!passwordInput.value) {
    showError("Parolni kiriting");
    return;
  }

  showError("");
  submitButton.disabled = true;
  submitButton.textContent = "Kirilmoqda...";

  try {
    const token = await login(`998${digits}`, passwordInput.value);
    if (!token) throw new Error("Server token qaytarmadi");
    await invoke("validate_and_open", { token });
  } catch (error) {
    if (
      [
        "storeNotAssigned",
        "terminalNotAssigned",
        "eposUnavailable",
        "terminalNotFound",
        "terminalMismatch",
      ].includes(error?.code)
    ) {
      showBlocked(error);
    } else {
      showError(
        error?.message ||
          (typeof error === "string" ? error : "Tizimga kirib bo‘lmadi"),
        error?.diagnostics || null,
      );
      submitButton.disabled = false;
      submitButton.textContent = "Kirish";
    }
  }
});
