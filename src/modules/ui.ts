export const elements = {
    get toolbar() { return document.getElementById("toolbar-container") },
    get changeFolderBtn() { return document.getElementById("change-folder-btn") },
    get selectFolderBtn() { return document.getElementById("select-folder") },
    get dropZone() { return document.getElementById("drop-zone") },
    get statusText() { return document.getElementById("status-text") },
    get app() { return document.getElementById("app") },
    get pathDisplay() { return document.getElementById("folder-path-display") },
};

export function updateStatus(message: string) {
    if (elements.statusText) {
        elements.statusText.textContent = message;
    }
}

export function toggleView(mode: 'import' | 'gallery') {
    if (!elements.dropZone || !elements.toolbar) return;

    if (mode === 'import') {
        elements.dropZone.style.display = 'flex';
        elements.dropZone.classList.remove("collapsed");
        const content = elements.dropZone.querySelector(".drop-zone-content") as HTMLElement;
        if (content) content.style.display = "block";

        elements.toolbar.classList.add('hidden');
        document.getElementById("gallery-section")?.remove();
        document.getElementById("duplicate-section")?.remove();

        if (elements.changeFolderBtn) elements.changeFolderBtn.style.display = "none";
    } else {
        elements.dropZone.style.display = 'none';
        elements.toolbar.classList.remove('hidden');
    }
}

export function showImportSuccess(imageCount: number, sizeBytes: number) {
    const sizeMB = (sizeBytes / 1024 / 1024).toFixed(2);
    updateStatus(`Fant ${imageCount} bilder (${sizeMB} MB)`);

    if (elements.dropZone) {
        elements.dropZone.classList.add("collapsed");
        const content = elements.dropZone.querySelector(".drop-zone-content") as HTMLElement;
        if (content) content.style.display = "none";
        if (elements.changeFolderBtn) elements.changeFolderBtn.style.display = "flex";
    }
}
