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

    function startDrag(clientX, clientY, target) {
      if (target && target.classList && target.classList.contains("desktop-close")) return false;
      dragging = true;
      const rect = win.getBoundingClientRect();
      startX = clientX;
      startY = clientY;
      startLeft = rect.left;
      startTop = rect.top;
      bumpZ(win);
      return true;
    }

    function moveDrag(clientX, clientY) {
      if (!dragging) return;
      win.style.left = (startLeft + clientX - startX) + "px";
      win.style.top = (startTop + clientY - startY) + "px";
    }

    function endDrag() { dragging = false; }

    handle.addEventListener("mousedown", function (e) {
      if (startDrag(e.clientX, e.clientY, e.target)) e.preventDefault();
    });
    document.addEventListener("mousemove", function (e) {
      moveDrag(e.clientX, e.clientY);
    });
    document.addEventListener("mouseup", endDrag);

    handle.addEventListener("touchstart", function (e) {
      if (e.touches.length !== 1) return;
      const t = e.touches[0];
      if (startDrag(t.clientX, t.clientY, e.target)) e.preventDefault();
    }, { passive: false });
    document.addEventListener("touchmove", function (e) {
      if (!dragging || e.touches.length !== 1) return;
      const t = e.touches[0];
      moveDrag(t.clientX, t.clientY);
      e.preventDefault();
    }, { passive: false });
    document.addEventListener("touchend", endDrag);
    document.addEventListener("touchcancel", endDrag);
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

  function attachResize(win, handle, onEnd) {
    let resizing = false;
    let startX = 0, startY = 0, startW = 0, startH = 0;
    const MIN_W = 200, MIN_H = 120;

    function startResize(clientX, clientY) {
      resizing = true;
      const rect = win.getBoundingClientRect();
      startX = clientX;
      startY = clientY;
      startW = rect.width;
      startH = rect.height;
      bumpZ(win);
    }

    function moveResize(clientX, clientY) {
      if (!resizing) return;
      win.style.width = Math.max(MIN_W, startW + clientX - startX) + "px";
      win.style.height = Math.max(MIN_H, startH + clientY - startY) + "px";
    }

    function endResize() {
      if (!resizing) return;
      resizing = false;
      if (onEnd && (win.offsetWidth !== startW || win.offsetHeight !== startH)) {
        onEnd(win);
      }
    }

    handle.addEventListener("mousedown", function (e) {
      startResize(e.clientX, e.clientY);
      e.preventDefault();
      e.stopPropagation();
    });
    document.addEventListener("mousemove", function (e) {
      moveResize(e.clientX, e.clientY);
    });
    document.addEventListener("mouseup", endResize);

    handle.addEventListener("touchstart", function (e) {
      if (e.touches.length !== 1) return;
      const t = e.touches[0];
      startResize(t.clientX, t.clientY);
      e.preventDefault();
      e.stopPropagation();
    }, { passive: false });
    document.addEventListener("touchmove", function (e) {
      if (!resizing || e.touches.length !== 1) return;
      const t = e.touches[0];
      moveResize(t.clientX, t.clientY);
      e.preventDefault();
    }, { passive: false });
    document.addEventListener("touchend", endResize);
    document.addEventListener("touchcancel", endResize);
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

    const resizeHandle = document.createElement("div");
    resizeHandle.className = "desktop-resize-handle";
    win.appendChild(resizeHandle);

    root.appendChild(win);

    attachDrag(win, titlebar);
    attachFocus(win);
    attachClose(win, closeBtn);
    attachResize(win, resizeHandle);
  }

  ready(function () {
    const terminal = document.querySelector('.desktop-window[data-window-id="terminal"]');
    if (terminal) {
      const titlebar = terminal.querySelector(".desktop-titlebar");
      if (titlebar) attachDrag(terminal, titlebar);
      attachFocus(terminal);
      bumpZ(terminal);
    }

    let resizeTimer;
    window.addEventListener("resize", function () {
      clearTimeout(resizeTimer);
      resizeTimer = setTimeout(function () { window.location.reload(); }, 200);
    });
  });

  // Expose to wasm-bindgen
  window.morisobaDesktop = { spawn_image: spawn_image };
})();
