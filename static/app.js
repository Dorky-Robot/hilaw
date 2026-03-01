(function () {
  "use strict";

  const PAGE_SIZE = 100;
  let devices = [];
  let currentDevice = null;
  let currentDir = null;
  let currentPath = "";
  let entries = [];
  let mediaEntries = []; // only images/raw/video for lightbox navigation
  let offset = 0;
  let hasMore = false;
  let loading = false;
  let lbIndex = -1;

  const $ = (sel) => document.querySelector(sel);
  const grid = $("#grid");
  const deviceList = $("#device-list");
  const breadcrumb = $("#breadcrumb");
  const lightbox = $("#lightbox");
  const lbContent = $("#lb-content");
  const lbInfo = $("#lb-info");
  const sentinel = $("#sentinel");

  // --- Sidebar ---
  $("#sidebar-toggle").onclick = () => $("#sidebar").classList.add("open");
  $("#sidebar-close").onclick = () => $("#sidebar").classList.remove("open");

  async function loadDevices() {
    try {
      const res = await fetch("/api/devices");
      devices = await res.json();
      renderSidebar();
      // Auto-select first device with directories
      const first = devices.find((d) => d.directories.length > 0);
      if (first) {
        selectDir(first.id, first.directories[0], "");
      }
    } catch (e) {
      deviceList.innerHTML =
        '<div class="loading">Failed to load devices</div>';
    }
  }

  function renderSidebar() {
    deviceList.innerHTML = "";
    for (const dev of devices) {
      const group = document.createElement("div");
      group.className = "device-group";

      const name = document.createElement("div");
      name.className = "device-name";
      name.textContent = dev.name + (dev.is_self ? " (local)" : "");
      group.appendChild(name);

      for (const dir of dev.directories) {
        const btn = document.createElement("button");
        btn.className = "dir-btn";
        btn.textContent = dir;
        btn.dataset.device = dev.id;
        btn.dataset.dir = dir;
        btn.onclick = () => {
          selectDir(dev.id, dir, "");
          $("#sidebar").classList.remove("open");
        };
        group.appendChild(btn);
      }

      deviceList.appendChild(group);
    }
  }

  function updateActiveDir() {
    document.querySelectorAll(".dir-btn").forEach((btn) => {
      btn.classList.toggle(
        "active",
        btn.dataset.device === currentDevice && btn.dataset.dir === currentDir
      );
    });
  }

  function selectDir(device, dir, path) {
    currentDevice = device;
    currentDir = dir;
    currentPath = path;
    offset = 0;
    entries = [];
    mediaEntries = [];
    updateActiveDir();
    updateBreadcrumb();
    grid.innerHTML = "";
    loadFiles();
  }

  // --- Breadcrumb ---
  function updateBreadcrumb() {
    breadcrumb.innerHTML = "";
    const dev = devices.find((d) => d.id === currentDevice);
    const devName = dev ? dev.name : currentDevice;

    addCrumb(devName + " / " + currentDir, () =>
      selectDir(currentDevice, currentDir, "")
    );

    if (currentPath) {
      const parts = currentPath.split("/");
      let accumulated = "";
      for (let i = 0; i < parts.length; i++) {
        addSep();
        accumulated += (i > 0 ? "/" : "") + parts[i];
        const p = accumulated;
        const isLast = i === parts.length - 1;
        addCrumb(parts[i], isLast ? null : () => selectDir(currentDevice, currentDir, p));
      }
    }
  }

  function addCrumb(text, onClick) {
    const span = document.createElement("span");
    span.textContent = text;
    if (onClick) span.onclick = onClick;
    else span.style.color = "var(--text)";
    breadcrumb.appendChild(span);
  }

  function addSep() {
    const sep = document.createElement("span");
    sep.className = "sep";
    sep.textContent = "/";
    breadcrumb.appendChild(sep);
  }

  // --- File Grid ---
  async function loadFiles() {
    if (loading) return;
    loading = true;

    try {
      const params = new URLSearchParams({
        device: currentDevice,
        dir: currentDir,
        path: currentPath,
        offset: offset.toString(),
        limit: PAGE_SIZE.toString(),
      });

      const res = await fetch("/api/browse?" + params);
      const data = await res.json();

      hasMore = data.has_more;
      offset += data.entries.length;

      for (const entry of data.entries) {
        entries.push(entry);
        if (!entry.is_dir && entry.file_type !== "other") {
          mediaEntries.push(entry);
        }
        grid.appendChild(createGridItem(entry));
      }
    } catch (e) {
      grid.innerHTML += '<div class="loading">Failed to load files</div>';
    }

    loading = false;
  }

  function createGridItem(entry) {
    const div = document.createElement("div");
    div.className = "grid-item";

    if (entry.is_dir) {
      div.innerHTML =
        '<div class="placeholder"><div class="folder-icon">&#128193;</div></div>';
      div.onclick = () =>
        selectDir(currentDevice, currentDir, entry.path);
    } else if (entry.file_type === "video") {
      div.innerHTML = '<div class="placeholder"></div><div class="video-badge">&#9654;</div>';
      div.onclick = () => openLightbox(entry);
    } else if (entry.file_type === "raw" || entry.file_type === "image") {
      const img = document.createElement("img");
      img.loading = "lazy";
      img.src = `/api/thumbnail?device=${enc(currentDevice)}&dir=${enc(currentDir)}&path=${enc(entry.path)}&w=300&h=300`;
      img.alt = entry.name;
      img.onerror = () => {
        img.replaceWith(
          Object.assign(document.createElement("div"), {
            className: "placeholder",
            textContent: entry.name,
          })
        );
      };
      div.appendChild(img);
      div.onclick = () => openLightbox(entry);
    } else {
      div.innerHTML = `<div class="placeholder">${escHtml(entry.name)}</div>`;
    }

    const label = document.createElement("div");
    label.className = "label";
    label.textContent = entry.name;
    div.appendChild(label);

    return div;
  }

  // --- Infinite Scroll ---
  const observer = new IntersectionObserver(
    (es) => {
      if (es[0].isIntersecting && hasMore && !loading) {
        loadFiles();
      }
    },
    { rootMargin: "200px" }
  );
  observer.observe(sentinel);

  // --- Lightbox ---
  function openLightbox(entry) {
    lbIndex = mediaEntries.findIndex(
      (e) => e.path === entry.path && e.name === entry.name
    );
    showLightboxEntry(entry);
    lightbox.classList.remove("hidden");
  }

  function showLightboxEntry(entry) {
    lbContent.innerHTML = "";
    lbInfo.textContent = entry.name;

    if (entry.file_type === "video") {
      const video = document.createElement("video");
      video.controls = true;
      video.autoplay = true;
      video.src = `/api/stream?device=${enc(currentDevice)}&dir=${enc(currentDir)}&path=${enc(entry.path)}`;
      lbContent.appendChild(video);
    } else {
      const img = document.createElement("img");
      img.src = `/api/preview?device=${enc(currentDevice)}&dir=${enc(currentDir)}&path=${enc(entry.path)}`;
      img.alt = entry.name;
      lbContent.appendChild(img);
    }
  }

  function closeLightbox() {
    lightbox.classList.add("hidden");
    lbContent.innerHTML = "";
    lbIndex = -1;
  }

  function lbNav(delta) {
    if (mediaEntries.length === 0) return;
    lbIndex = (lbIndex + delta + mediaEntries.length) % mediaEntries.length;
    showLightboxEntry(mediaEntries[lbIndex]);
  }

  $("#lb-close").onclick = closeLightbox;
  $("#lb-prev").onclick = () => lbNav(-1);
  $("#lb-next").onclick = () => lbNav(1);

  lightbox.addEventListener("click", (e) => {
    if (e.target === lightbox) closeLightbox();
  });

  document.addEventListener("keydown", (e) => {
    if (lightbox.classList.contains("hidden")) return;
    if (e.key === "Escape") closeLightbox();
    if (e.key === "ArrowLeft") lbNav(-1);
    if (e.key === "ArrowRight") lbNav(1);
  });

  // Touch swipe in lightbox
  let touchStartX = 0;
  lightbox.addEventListener("touchstart", (e) => {
    touchStartX = e.touches[0].clientX;
  });
  lightbox.addEventListener("touchend", (e) => {
    const dx = e.changedTouches[0].clientX - touchStartX;
    if (Math.abs(dx) > 50) {
      lbNav(dx > 0 ? -1 : 1);
    }
  });

  // --- Helpers ---
  function enc(s) {
    return encodeURIComponent(s);
  }

  function escHtml(s) {
    const d = document.createElement("div");
    d.textContent = s;
    return d.innerHTML;
  }

  // --- Init ---
  loadDevices();
})();
