// MORISOBA brutalist desktop window manager — vanilla JS, no dependencies.
// Wires window dragging, focus stacking, [X] closing, and an image-spawn
// API that the WASM side calls via wasm-bindgen.

(function () {
  "use strict";

  const STATE = {
    zCounter: 100,
    cascade: 0,
    // Cascade base offset (relative to #desktop-root): drives where new
    // image windows appear so each spawn is visible next to the previous.
    cascadeBase: { x: 80, y: 64 },
    cascadeStep: 28,
    cascadeWrap: 8,
  };

  function ready(fn) {
    if (document.readyState !== "loading") fn();
    else document.addEventListener("DOMContentLoaded", fn);
  }

  function bumpZ(el) {
    STATE.zCounter += 1;
    el.style.zIndex = String(STATE.zCounter);
  }

  function attachDrag(win, handle) {
    let dragging = false;
    let startX = 0, startY = 0, startLeft = 0, startTop = 0;
    handle.addEventListener("mousedown", function (e) {
      // Ignore clicks on the close button — let it fire its own handler
      if (e.target && e.target.classList && e.target.classList.contains("desktop-close")) return;
      dragging = true;
      const rect = win.getBoundingClientRect();
      startX = e.clientX;
      startY = e.clientY;
      startLeft = rect.left;
      startTop = rect.top;
      bumpZ(win);
      e.preventDefault();
    });
    document.addEventListener("mousemove", function (e) {
      if (!dragging) return;
      const dx = e.clientX - startX;
      const dy = e.clientY - startY;
      win.style.left = (startLeft + dx) + "px";
      win.style.top = (startTop + dy) + "px";
    });
    document.addEventListener("mouseup", function () {
      dragging = false;
    });
  }

  function attachFocus(win) {
    win.addEventListener("mousedown", function () { bumpZ(win); });
  }

  function attachClose(win, closeBtn) {
    closeBtn.addEventListener("click", function (e) {
      e.stopPropagation();
      const url = closeBtn.dataset.blobUrl;
      if (url) URL.revokeObjectURL(url);
      win.remove();
    });
  }

  function nextCascadePosition() {
    const slot = STATE.cascade % STATE.cascadeWrap;
    STATE.cascade += 1;
    return {
      x: STATE.cascadeBase.x + slot * STATE.cascadeStep,
      y: STATE.cascadeBase.y + slot * STATE.cascadeStep,
    };
  }

  function escapeText(s) {
    const div = document.createElement("div");
    div.textContent = String(s);
    return div.innerHTML;
  }

  function spawn_image(bytes, slug) {
    // bytes is a Uint8Array from wasm-bindgen; slug is the item slug
    const blob = new Blob([bytes], { type: "image/png" });
    const url = URL.createObjectURL(blob);
    const safeSlug = escapeText(slug || "image");

    const root = document.getElementById("desktop-root");
    if (!root) {
      console.warn("morisobaDesktop: #desktop-root missing; cannot spawn image");
      return;
    }

    const pos = nextCascadePosition();
    const win = document.createElement("div");
    win.className = "desktop-window image-window";
    win.dataset.windowId = "image-" + STATE.cascade;
    win.style.left = pos.x + "px";
    win.style.top = pos.y + "px";
    bumpZ(win);

    const titlebar = document.createElement("div");
    titlebar.className = "desktop-titlebar";

    const title = document.createElement("span");
    title.className = "desktop-title";
    title.innerHTML = "[ IMAGE :: " + safeSlug + " ]";

    const closeBtn = document.createElement("button");
    closeBtn.className = "desktop-close";
    closeBtn.type = "button";
    closeBtn.textContent = "[X]";
    closeBtn.dataset.blobUrl = url;

    titlebar.appendChild(title);
    titlebar.appendChild(closeBtn);

    const body = document.createElement("div");
    body.className = "desktop-body";

    const img = document.createElement("img");
    img.src = url;
    img.alt = safeSlug;

    body.appendChild(img);
    win.appendChild(titlebar);
    win.appendChild(body);
    root.appendChild(win);

    attachDrag(win, titlebar);
    attachFocus(win);
    attachClose(win, closeBtn);
  }

  ready(function () {
    // Attach drag + focus to the initial terminal window so it can also be moved.
    const terminal = document.querySelector('.desktop-window[data-window-id="terminal"]');
    if (terminal) {
      const titlebar = terminal.querySelector(".desktop-titlebar");
      if (titlebar) attachDrag(terminal, titlebar);
      attachFocus(terminal);
      bumpZ(terminal);
    }
  });

  // Expose to wasm-bindgen
  window.morisobaDesktop = { spawn_image: spawn_image };
})();