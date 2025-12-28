import { invoke } from "@tauri-apps/api/core";
import { toast } from "./toast";
import { convertFileSrc } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { DuplicateResult, DuplicateGroup, ImageInfo, OperationResult } from "./types";
import { state, CONFIG } from "./state";
import { updateStatus } from "./ui";
import { comparisonManager } from "./comparison";

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
    // const original = group.images[0];

    group.images.forEach((img, index) => {
        const item = createDuplicateItem(img, index === 0);
        groupGrid.appendChild(item);
    });

    groupDiv.appendChild(groupHeader);
    groupDiv.appendChild(groupGrid);

    // Bind Compare Button (prevent collapse when clicking button)
    const compareBtn = groupHeader.querySelector(".compare-btn");
    compareBtn?.addEventListener("click", (e) => {
        e.stopPropagation(); // Don't toggle collapse
        comparisonManager.open(group);
    });

    // Toggle Collapse
    groupHeader.addEventListener("click", () => {
        groupDiv.classList.toggle("collapsed");
    });

    // Default to collapsed? Or open? User asked for "collapse groups to make it tidier".
    // Let's default to OPEN but allow collapse, or maybe collapse if many groups?
    // Let's default to OPEN but styled nicely, unless user explicit asked for default collapsed.
    // "Kan vi kollapse duplikatgrupper for at det skal se ryddigere ut?" -> impl: clickable header.
    // Let's start expanded.

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




async function deleteSelectedDuplicates(section: HTMLElement) {
    const checkboxes = section.querySelectorAll(".gallery-checkbox:checked") as NodeListOf<HTMLInputElement>;
    const selectedPaths = Array.from(checkboxes).map(cb => cb.dataset.path).filter(p => p !== undefined) as string[];

    if (selectedPaths.length === 0) return;

    if (!confirm(`Vil du slette ${selectedPaths.length} duplikater?`)) return;

    try {
        const result = await invoke<OperationResult>("delete_images", { paths: selectedPaths });
        toast.show(`Slettet ${result.success} bilder.`, "success");

        selectedPaths.forEach(path => {
            const item = section.querySelector(`.gallery-item[data-path="${CSS.escape(path)}"]`);
            item?.remove();
        });
    } catch (error) {
        toast.show(`Feil: ${error}`, "error");
    }
}
