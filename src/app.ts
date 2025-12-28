import { invoke } from "@tauri-apps/api/core";
import { open } from '@tauri-apps/plugin-dialog';
// Scanner logic is implemented in performScan below
// Actually I'll implement scanFolder here.

import { state } from "./modules/state";
import { ScanResult } from "./modules/types";
import { elements, updateStatus, showImportSuccess, toggleView } from "./modules/ui";
import { initGallery } from "./modules/gallery";
import { renderVirtualItems } from "./modules/virtual-scroll";

export function setupApp() {
    // Event Listeners for Import View
    elements.selectFolderBtn?.addEventListener("click", async () => {
        try {
            const selected = await open({
                directory: true,
                multiple: false,
                title: "Velg mappe med bilder",
            });

            if (selected) {
                const path = Array.isArray(selected) ? selected[0] : selected;
                if (elements.pathDisplay) elements.pathDisplay.textContent = path;

                toggleView('gallery');
                updateStatus("Skanner mappe...");
                await performScan(path);
            }
        } catch (error) {
            console.error("Feil ved valg av mappe:", error);
            updateStatus(`Feil ved valg av mappe: ${error}`);
            toggleView('import');
        }
    });

    elements.changeFolderBtn?.addEventListener("click", () => {
        toggleView('import');
        updateStatus("Velg en mappe for å starte");
    });

    // Drag and Drop (Basic)
    elements.dropZone?.addEventListener("dragover", (e) => {
        e.preventDefault();
        elements.dropZone?.classList.add("drag-over");
    });

    elements.dropZone?.addEventListener("dragleave", () => {
        elements.dropZone?.classList.remove("drag-over");
    });

    elements.dropZone?.addEventListener("drop", async (e) => {
        e.preventDefault();
        elements.dropZone?.classList.remove("drag-over");
        // TODO: Implement actual file handling from drop
        updateStatus("Dra-og-slipp støttes snart");
    });

    // Keyboard Shortcuts
    document.addEventListener("keydown", (e) => {
        // ESC: Close overlays or clear selection
        if (e.key === "Escape") {
            const modals = document.querySelectorAll(".modal-overlay.open");
            if (modals.length > 0) {
                modals.forEach(m => m.remove()); // Or classList.remove generic
            } else {
                state.clearSelection();
                renderVirtualItems();
            }
        }

        // Ctrl+A: Select All
        if ((e.ctrlKey || e.metaKey) && e.key === "a") {
            e.preventDefault();
            document.getElementById("select-all")?.click();
        }

        // Delete: Delete selected
        if (e.key === "Delete") {
            if (state.selectedPaths.size > 0) {
                document.getElementById("delete-selected")?.click();
            }
        }
    });
}

async function performScan(path: string) {
    try {
        const result = await invoke<ScanResult>("scan_folder", { path });

        state.setImages(result.images);
        state.clearSelection();

        showImportSuccess(result.imageCount, result.totalSizeBytes);
        initGallery();

    } catch (error) {
        console.error("Feil ved skanning:", error);
        updateStatus(`Feil: ${error}`);
    }
}
