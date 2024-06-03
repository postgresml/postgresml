function createToast(message) {
  const toastElement = document.createElement("div");
  toastElement.classList.add("toast", "hide");
  toastElement.setAttribute("role", "alert");
  toastElement.setAttribute("aria-live", "assertive");
  toastElement.setAttribute("aria-atomic", "true");

  const toastBodyElement = document.createElement("div");
  toastBodyElement.classList.add("toast-body");
  toastBodyElement.innerHTML = message;

  toastElement.appendChild(toastBodyElement);

  const container = document.getElementById("toast-container");

  if (container) {
    container.appendChild(toastElement);

    // remove from DOM when no longer needed
    toastElement.addEventListener("hidden.bs.toast", (e) => e.target.remove());

    return toastElement;
  } else {
    return null;
  }
}

function showToast(toastElement, config) {
  config = config || {
    autohide: true,
    delay: 2000,
  };
  const toastBootstrap = bootstrap.Toast.getOrCreateInstance(
    toastElement,
    config,
  );
  toastBootstrap.show();
}

export { createToast, showToast };
