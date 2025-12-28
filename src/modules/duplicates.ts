import { invoke } from "@tauri-apps/api/core";
import { convertFileSrc } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { DuplicateResult, DuplicateGroup, ImageInfo, OperationResult } from "./types";
import { state, CONFIG } from "./state";
import { updateStatus } from "./ui";

export async function startDuplicateDetection() {
    if (state.currentImages.length === 0) {
        updateStatus("Velg en mappe først");
        return;
    }

    const btn = document.getElementById("find-duplicates");
    try {
        btn?.classList.add("loading");
        updateStatus(`Analyserer ${state.currentImages.length} bilder...`);

        const paths = state.currentImages.map((img) => img.path);

        let processedCount = 0;
        const unlisten = await listen("progress", () => {
            processedCount++;
            updateStatus(`Analyserer ${processedCount}/${paths.length} bilder...`);
        });

        const result = await invoke<DuplicateResult>("find_duplicates", {
            paths,
            threshold: CONFIG.DUPLICATE_THRESHOLD,
        });

        unlisten();

        if (result.totalDuplicates === 0) {
            updateStatus(`Ingen duplikater funnet (${result.processed} bilder)`);
        } else {
            updateStatus(
                `Fant ${result.totalDuplicates} duplikater i ${result.groups.length} grupper`
            );
            displayDuplicates(result.groups);
        }

    } catch (error) {
        console.error("Feil ved duplikatdeteksjon:", error);
        updateStatus(`Feil: ${error}`);
    } finally {
        btn?.classList.remove("loading");
    }
}

function displayDuplicates(groups: DuplicateGroup[]) {
    const app = document.getElementById("app");
    if (!app) return;

    document.getElementById("duplicate-section")?.remove();

    const section = document.createElement("section");
    section.id = "duplicate-section";
    section.className = "duplicate-section";

    const header = document.createElement("div");
    header.className = "gallery-header";
    header.innerHTML = `
      <h2>🔍 Duplikatgrupper (${groups.length})</h2>
      <div class="duplicate-controls">
          <button class="btn btn-danger" id="delete-duplicates-btn">🗑️ Slett valgte</button>
          <button class="btn btn-secondary" id="close-duplicates">✕ Lukk</button>
      </div>
    `;

    const gallerySection = document.getElementById("gallery-section");
    if (gallerySection) gallerySection.style.display = "none";

    section.appendChild(header);

    // Render Groups
    groups.forEach((group, index) => {
        section.appendChild(createGroupElement(group, index));
    });

    const container = app.querySelector(".container");
    if (container) {
        container.appendChild(section);
    }

    section.scrollIntoView({ behavior: "smooth", block: "start" });

    // Events
    document.getElementById("close-duplicates")?.addEventListener("click", () => {
        section.remove();
        if (gallerySection) gallerySection.style.display = "block";
    });

    document.getElementById("delete-duplicates-btn")?.addEventListener("click", () => deleteSelectedDuplicates(section));
}

function createGroupElement(group: DuplicateGroup, groupIndex: number): HTMLElement {
    const groupDiv = document.createElement("div");
    groupDiv.className = "duplicate-group";
    groupDiv.dataset.groupIndex = String(groupIndex);

    const groupHeader = document.createElement("div");
    groupHeader.className = "duplicate-group-header";
    groupHeader.innerHTML = `
        <span>Gruppe ${groupIndex + 1} (${group.images.length} bilder)</span>
        <button class="btn btn-sm btn-secondary compare-btn">⚖️ Sammenlign</button>
    `;

    const groupGrid = document.createElement("div");
    groupGrid.className = "duplicate-grid";

    // Original is usually first
    const original = group.images[0];

    group.images.forEach((img, index) => {
        const item = createDuplicateItem(img, index === 0);
        groupGrid.appendChild(item);
    });

    groupDiv.appendChild(groupHeader);
    groupDiv.appendChild(groupGrid);

    // Bind Compare Button
    groupHeader.querySelector(".compare-btn")?.addEventListener("click", () => {
        // Compare first duplicate with original
        if (group.images.length > 1) {
            showSideBySide(original, group.images[1], group.images.slice(2));
        }
    });

    return groupDiv;
}

function createDuplicateItem(img: ImageInfo, isOriginal: boolean): HTMLElement {
    const item = document.createElement("div");
    item.className = `gallery-item ${isOriginal ? 'original' : 'duplicate'}`;
    item.dataset.path = img.path;

    const sizeKB = (img.sizeBytes / 1024).toFixed(1);

    item.innerHTML = `
      <div class="gallery-item-image">
        <img src="${convertFileSrc(img.path)}" loading="lazy" />
        <div class="gallery-item-overlay">
           ${!isOriginal ? `<input type="checkbox" class="gallery-checkbox" data-path="${img.path}">` : ''}
        </div>
      </div>
      <div class="gallery-item-info">
        <span class="gallery-item-name" title="${img.filename}">${img.filename}</span>
        <span class="gallery-item-size">${sizeKB} KB</span>
      </div>
    `;

    if (!isOriginal) {
        item.addEventListener("click", (e) => {
            const cb = item.querySelector("input") as HTMLInputElement;
            if (e.target !== cb) cb.checked = !cb.checked;
            if (cb.checked) item.classList.add("selected");
            else item.classList.remove("selected");
        });
    }

    return item;
}

// Side by Side Comparison
function showSideBySide(original: ImageInfo, candidate: ImageInfo, others: ImageInfo[]) {
    const overlay = document.createElement("div");
    overlay.className = "modal-overlay open";

    // Simple navigation if others exist

    const updateContent = (cand: ImageInfo) => {
        const content = overlay.querySelector(".compare-content");
        if (!content) return;

        content.innerHTML = `
            <div class="compare-card original">
                <h3>Original</h3>
                <div class="compare-img">
                    <img src="${convertFileSrc(original.path)}">
                </div>
                <div class="compare-meta">
                    <p>${original.filename}</p>
                    <p>${(original.sizeBytes / 1024).toFixed(1)} KB</p>
                </div>
            </div>
            <div class="compare-card duplicate">
                <h3>Duplikat</h3>
                <div class="compare-img">
                    <img src="${convertFileSrc(cand.path)}">
                </div>
                <div class="compare-meta">
                    <p>${cand.filename}</p>
                    <p>${(cand.sizeBytes / 1024).toFixed(1)} KB</p>
                </div>
                <div class="compare-actions">
                     <button class="btn btn-danger delete-cand-btn">Slett Duplikat</button>
                     <button class="btn btn-secondary next-btn" ${others.length === 0 ? 'disabled' : ''}>Neste 👉</button>
                </div>
            </div>
        `;

        content.querySelector(".delete-cand-btn")?.addEventListener("click", async () => {
            if (confirm("Slett dette duplikatet?")) {
                await deleteImage(cand.path);
                // Move to next or close
                if (others.length > 0) {
                    updateContent(others.shift()!);
                } else {
                    overlay.remove();
                    // Refresh list?
                    const item = document.querySelector(`.gallery-item[data-path="${CSS.escape(cand.path)}"]`);
                    item?.remove();
                }
            }
        });

        content.querySelector(".next-btn")?.addEventListener("click", () => {
            if (others.length > 0) {
                updateContent(others.shift()!);
            }
        });
    };

    overlay.innerHTML = `
        <div class="modal compare-modal">
            <div class="modal-header">
                <h3>Sammenlign</h3>
                <button class="btn-close">✕</button>
            </div>
            <div class="modal-content compare-content">
                <!-- Injected js -->
            </div>
        </div>
    `;

    document.body.appendChild(overlay);
    updateContent(candidate);

    overlay.querySelector(".btn-close")?.addEventListener("click", () => overlay.remove());
}

async function deleteImage(path: string) {
    try {
        await invoke("delete_images", { paths: [path] });
    } catch (e) {
        alert(e);
    }
}

async function deleteSelectedDuplicates(section: HTMLElement) {
    const checkboxes = section.querySelectorAll(".gallery-checkbox:checked") as NodeListOf<HTMLInputElement>;
    const selectedPaths = Array.from(checkboxes).map(cb => cb.dataset.path).filter(p => p !== undefined) as string[];

    if (selectedPaths.length === 0) return;

    if (!confirm(`Vil du slette ${selectedPaths.length} duplikater?`)) return;

    try {
        const result = await invoke<OperationResult>("delete_images", { paths: selectedPaths });
        alert(`Slettet ${result.success} bilder.`);

        selectedPaths.forEach(path => {
            const item = section.querySelector(`.gallery-item[data-path="${CSS.escape(path)}"]`);
            item?.remove();
        });
    } catch (error) {
        alert(`Feil: ${error}`);
    }
}
