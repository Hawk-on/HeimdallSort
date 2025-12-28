import { invoke } from "@tauri-apps/api/core";
import { convertFileSrc } from "@tauri-apps/api/core";
import { open } from '@tauri-apps/plugin-dialog';
import { ImageInfo, OperationResult } from "./types";
import { state } from "./state";
import { setupVirtualScroll, renderVirtualItems } from "./virtual-scroll";
import { startDuplicateDetection } from "./duplicates"; // Forward reference
import { updateStatus } from "./ui";

let scrollContainer: HTMLDivElement | null = null;
let spacer: HTMLDivElement | null = null;
let scrollContent: HTMLDivElement | null = null;

export function initGallery() {
    const app = document.getElementById("app");
    if (!app) return;

    // Clean up
    document.getElementById("gallery-section")?.remove();
    document.getElementById("duplicate-section")?.remove();

    if (state.currentImages.length === 0) return;

    const gallerySection = document.createElement("section");
    gallerySection.id = "gallery-section";
    gallerySection.className = "gallery-section";

    // Header
    const galleryHeader = document.createElement("div");
    galleryHeader.className = "gallery-header";
    galleryHeader.id = "gallery-header";
    updateGalleryHeader(galleryHeader);

    // Virtual Scroll Container
    scrollContainer = document.createElement("div");
    scrollContainer.className = "virtual-scroll-container";

    spacer = document.createElement("div");
    spacer.className = "virtual-scroll-spacer";
    scrollContainer.appendChild(spacer);

    scrollContent = document.createElement("div");
    scrollContent.className = "virtual-scroll-content";
    scrollContainer.appendChild(scrollContent);

    gallerySection.appendChild(galleryHeader);
    gallerySection.appendChild(scrollContainer);

    const container = app.querySelector(".container");
    if (container) {
        container.appendChild(gallerySection);
    }

    // Attach listeners
    document.getElementById("find-duplicates")?.addEventListener("click", startDuplicateDetection);
    document.getElementById("sort-images")?.addEventListener("click", sortImages);
    document.getElementById("select-all")?.addEventListener("click", toggleSelectAll);
    document.getElementById("delete-selected")?.addEventListener("click", deleteSelected);
    document.getElementById("move-selected")?.addEventListener("click", moveSelected);

    // Init virtual scroll
    setupVirtualScroll(scrollContainer, spacer, scrollContent, createGalleryItem);
}

function updateGalleryHeader(header: HTMLElement) {
    header.innerHTML = `
      <h2>📷 Bilder (${state.currentImages.length})</h2>
      <div class="gallery-controls">
        <button class="btn btn-accent" id="find-duplicates">🔍 Finn duplikater</button>
        <button class="btn btn-primary" id="sort-images">📂 Sorter Alt</button>
        <div class="divider"></div>
        <button class="btn btn-secondary" id="select-all">Velg alle</button>
        <button class="btn btn-danger" id="delete-selected">🗑️ Slett valgte</button>
        <button class="btn btn-secondary" id="move-selected">➡️ Flytt valgte</button>
      </div>
    `;
}

export function createGalleryItem(img: ImageInfo, index: number): HTMLDivElement {
    const item = document.createElement("div");
    item.className = "gallery-item";
    item.dataset.index = String(index);
    item.dataset.path = img.path;

    const sizeKB = (img.sizeBytes / 1024).toFixed(1);

    const checkbox = document.createElement("input");
    checkbox.type = "checkbox";
    checkbox.className = "gallery-checkbox";
    checkbox.dataset.path = img.path;

    if (state.selectedPaths.has(img.path)) {
        checkbox.checked = true;
        item.classList.add("selected");
    }

    item.innerHTML = `
      <div class="gallery-item-image">
        <div class="thumbnail-placeholder"></div>
        <img src="" alt="${img.filename}" style="display: none;" />
        <div class="gallery-item-overlay">
        </div>
      </div>
      <div class="gallery-item-info">
        <span class="gallery-item-name" title="${img.filename}">${img.filename}</span>
        <span class="gallery-item-size">${sizeKB} KB</span>
      </div>
    `;
    item.querySelector(".gallery-item-overlay")?.appendChild(checkbox);

    loadThumbnail(item, img.path);

    item.addEventListener("click", (e) => {
        if ((e.target as HTMLElement) !== checkbox) {
            checkbox.checked = !checkbox.checked;
        }
        if (checkbox.checked) {
            state.addToSelection(img.path);
            item.classList.add("selected");
        } else {
            state.removeFromSelection(img.path);
            item.classList.remove("selected");
        }
    });

    item.addEventListener("dblclick", async (e) => {
        e.preventDefault();
        e.stopPropagation();
        try {
            await invoke("open_image", { path: img.path });
        } catch (error) {
            console.error("Kunne ikke åpne bilde:", error);
        }
    });

    return item;
}

async function loadThumbnail(item: HTMLDivElement, imagePath: string) {
    const imgElement = item.querySelector("img") as HTMLImageElement;
    const placeholder = item.querySelector(".thumbnail-placeholder") as HTMLElement;

    if (!imgElement) return;

    // Check cache first
    if (state.thumbnailCache.has(imagePath)) {
        const cachedSrc = state.thumbnailCache.get(imagePath)!;
        imgElement.src = cachedSrc;
        imgElement.style.display = "block";
        if (placeholder) placeholder.style.display = "none";
        return;
    }

    try {
        const thumbnailPath = await invoke<string>("get_thumbnail", { path: imagePath });

        if (thumbnailPath) {
            const src = convertFileSrc(thumbnailPath);
            // Update cache
            state.thumbnailCache.set(imagePath, src);

            // Double check if element is still valid/visible (virtual scroll recycling usually handles by creating new elements, but safe to check)
            if (imgElement) {
                imgElement.src = src;
                imgElement.style.display = "block";
                if (placeholder) placeholder.style.display = "none";
            }
        }
    } catch (error) {
        // Fallback to full image if thumbnail fails
        const src = convertFileSrc(imagePath);
        state.thumbnailCache.set(imagePath, src); // Cache the fallback too

        imgElement.src = src;
        imgElement.style.display = "block";
        if (placeholder) placeholder.style.display = "none";
    }
}

function toggleSelectAll() {
    if (state.selectedPaths.size === state.currentImages.length) {
        state.clearSelection();
    } else {
        state.currentImages.forEach(img => state.addToSelection(img.path));
    }
    renderVirtualItems();
}

// Actions

async function deleteSelected() {
    const selected = state.getSelectedImages();
    if (selected.length === 0) {
        alert("Ingen bilder valgt");
        return;
    }

    if (!confirm(`Er du sikker på at du vil slette ${selected.length} bilder?`)) {
        return;
    }

    const btn = document.getElementById("delete-selected");
    try {
        btn?.classList.add("loading");
        const paths = selected.map(img => img.path);
        const result = await invoke<OperationResult>("delete_images", { paths });

        let msg = `Slettet ${result.success} bilder.`;
        if (result.errors > 0) msg += ` ${result.errors} feil.`;

        updateStatus(msg);
        alert(msg);

        if (result.success > 0) {
            const newImages = state.currentImages.filter(img => !paths.includes(img.path));
            state.setImages(newImages);
            state.clearSelection();
            initGallery();
        }

    } catch (error) {
        console.error("Feil ved sletting:", error);
        alert(`Feil ved sletting: ${error}`);
    } finally {
        btn?.classList.remove("loading");
    }
}

async function moveSelected() {
    const selected = state.getSelectedImages();
    if (selected.length === 0) return;

    try {
        const targetDir = await open({
            directory: true,
            multiple: false,
            title: "Velg målmappe for flytting",
        });

        if (!targetDir) return;
        const targetPath = Array.isArray(targetDir) ? targetDir[0] : targetDir;

        const btn = document.getElementById("move-selected");
        btn?.classList.add("loading");
        updateStatus(`Flytter ${selected.length} bilder...`);

        const paths = selected.map(img => img.path);
        const result = await invoke<OperationResult>("move_images", { paths, targetDir: targetPath });

        let msg = `Flyttet ${result.success} bilder.`;
        if (result.errors > 0) msg += ` ${result.errors} feil.`;

        updateStatus(msg);
        alert(msg);

        if (result.success > 0) {
            const newImages = state.currentImages.filter(img => !paths.includes(img.path));
            state.setImages(newImages);
            state.clearSelection();
            initGallery();
        }

    } catch (error) {
        alert(`Feil ved flytting: ${error}`);
    } finally {
        document.getElementById("move-selected")?.classList.remove("loading");
    }
}

function createSortDialog(): Promise<{ confirmed: boolean; useDayFolder: boolean; useMonthNames: boolean } | null> {
    return new Promise((resolve) => {
        const overlay = document.createElement("div");
        overlay.className = "modal-overlay";
        overlay.innerHTML = `
            <div class="modal">
                <div class="modal-header">
                    <h3>Sorteringsvalg</h3>
                </div>
                <div class="modal-content">
                    <div class="form-group">
                        <label class="checkbox-label">
                            <input type="checkbox" id="sort-day-folder">
                            Opprett mappe for hver dag (År/Måned/Dag)
                        </label>
                    </div>
                    <div class="form-group">
                        <label class="checkbox-label">
                            <input type="checkbox" id="sort-month-names" checked>
                            Bruk månedsnavn (01 - Januar)
                        </label>
                    </div>
                </div>
                <div class="modal-footer">
                    <button class="btn btn-secondary" id="modal-cancel">Avbryt</button>
                    <button class="btn btn-primary" id="modal-confirm">Start Sortering</button>
                </div>
            </div>
        `;

        document.body.appendChild(overlay);
        requestAnimationFrame(() => overlay.classList.add("open"));

        const close = () => {
            overlay.classList.remove("open");
            setTimeout(() => overlay.remove(), 300);
        };

        document.getElementById("modal-cancel")?.addEventListener("click", () => {
            close();
            resolve(null);
        });

        document.getElementById("modal-confirm")?.addEventListener("click", () => {
            const useDayFolder = (document.getElementById("sort-day-folder") as HTMLInputElement).checked;
            const useMonthNames = (document.getElementById("sort-month-names") as HTMLInputElement).checked;
            close();
            resolve({ confirmed: true, useDayFolder, useMonthNames });
        });
    });
}

async function sortImages() {
    if (state.currentImages.length === 0) return;

    try {
        const targetDir = await open({
            directory: true,
            multiple: false,
            title: "Velg målmappe for sortering",
        });

        if (!targetDir) return;
        const targetPath = Array.isArray(targetDir) ? targetDir[0] : targetDir;

        const options = await createSortDialog();
        if (!options || !options.confirmed) return;

        updateStatus("Sorterer bilder...");
        const btn = document.getElementById("sort-images");
        btn?.classList.add("loading");

        const paths = state.currentImages.map((img) => img.path);
        const result = await invoke<OperationResult>("sort_images_by_date", {
            paths,
            method: "copy", // Default to copy for safety
            targetDir: targetPath,
            options: {
                useDayFolder: options.useDayFolder,
                useMonthNames: options.useMonthNames
            }
        });

        let message = `Sortering ferdig: ${result.success} kopiert, ${result.errors} feil.`;
        updateStatus(message);
        alert(message);

    } catch (error) {
        updateStatus(`Feil ved sortering: ${error}`);
    } finally {
        document.getElementById("sort-images")?.classList.remove("loading");
    }
}
