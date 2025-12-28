import { convertFileSrc } from "@tauri-apps/api/core";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "./toast";
import { ImageInfo, DuplicateGroup, OperationResult } from "./types";
import { updateStatus } from "./ui";

export class ComparisonManager {
    private overlay: HTMLElement | null = null;
    private remainingImages: ImageInfo[] = []; // Other duplicates in the group
    private originalImage: ImageInfo | null = null;
    private currentCandidate: ImageInfo | null = null;

    open(group: DuplicateGroup) {
        if (group.images.length < 2) return;

        // Assume first image is "original" or "best candidate" initially
        this.originalImage = group.images[0];
        // The rest are candidates
        this.remainingImages = group.images.slice(1);

        this.createOverlay();
        this.showNextCandidate();
    }

    private createOverlay() {
        this.overlay = document.createElement("div");
        this.overlay.className = "modal-overlay open comparison-overlay";
        this.overlay.innerHTML = `
            <div class="comparison-modal">
                <div class="comparison-header">
                    <h3>⚔️ Sammenlign Bilder</h3>
                    <div class="comparison-controls-hint">
                        <span>⬅️ Behold Venstre</span>
                        <span>➡️ Behold Høyre</span>
                        <span>❌ Slett begge</span>
                        <span>Esc Lukk</span>
                    </div>
                    <button class="btn-close">✕</button>
                </div>
                <div class="comparison-body">
                    <div class="comparison-side left" id="comp-left">
                        <!-- Injected -->
                    </div>
                    <div class="comparison-vs">VS</div>
                    <div class="comparison-side right" id="comp-right">
                        <!-- Injected -->
                    </div>
                </div>
                <div class="comparison-footer">
                    <button class="btn btn-danger" id="comp-delete-left">🗑️ Slett Venstre</button>
                    <button class="btn btn-primary" id="comp-keep-both">Behold Begge</button>
                    <button class="btn btn-danger" id="comp-delete-right">🗑️ Slett Høyre</button>
                </div>
            </div>
        `;

        document.body.appendChild(this.overlay);

        // Event Listeners
        this.overlay.querySelector(".btn-close")?.addEventListener("click", () => this.close());

        // Keyboard navigation
        document.addEventListener("keydown", this.handleKeydown);

        // Button Listeners
        document.getElementById("comp-delete-left")?.addEventListener("click", () => this.resolve("right")); // Keep right -> delete left
        document.getElementById("comp-delete-right")?.addEventListener("click", () => this.resolve("left")); // Keep left -> delete right
        document.getElementById("comp-keep-both")?.addEventListener("click", () => this.resolve("both"));
    }

    private handleKeydown = (e: KeyboardEvent) => {
        if (!this.overlay) return;

        switch (e.key) {
            case "ArrowLeft":
                this.resolve("left"); // Choose left (implies keeping left, maybe deleting right? logic TBD)
                // Let's define ArrowLeft as "Prefer Left Image" -> Delete Right
                break;
            case "ArrowRight":
                this.resolve("right"); // Prefer Right -> Delete Left
                break;
            case "Escape":
                this.close();
                break;
        }
    }

    private close() {
        if (this.overlay) {
            this.overlay.remove();
            this.overlay = null;
        }
        document.removeEventListener("keydown", this.handleKeydown);
        // Refresh duplicates view if needed?
    }

    private showNextCandidate() {
        if (this.remainingImages.length === 0) {
            this.close();
            updateStatus("Sammenligning ferdig for denne gruppen.");
            // Trigger refresh of main duplicate list? 
            // Ideally we should emit an event or callback.
            return;
        }

        this.currentCandidate = this.remainingImages[0];
        this.renderComparison();
    }

    private renderComparison() {
        if (!this.originalImage || !this.currentCandidate) return;

        const leftContainer = document.getElementById("comp-left");
        const rightContainer = document.getElementById("comp-right");

        if (leftContainer) leftContainer.innerHTML = this.createImageCard(this.originalImage, "Original");
        if (rightContainer) rightContainer.innerHTML = this.createImageCard(this.currentCandidate, "Kandidat");
    }

    private createImageCard(img: ImageInfo, label: string): string {
        const sizeKB = (img.sizeBytes / 1024).toFixed(1);
        return `
            <div class="comp-card">
                <div class="comp-label">${label}</div>
                <div class="comp-img-wrapper">
                    <img src="${convertFileSrc(img.path)}" class="comp-img">
                </div>
                <div class="comp-meta">
                    <div class="meta-row">📄 ${img.filename}</div>
                    <div class="meta-row">💾 ${sizeKB} KB</div>
                </div>
            </div>
        `;
    }

    private async resolve(decision: "left" | "right" | "both") {
        if (!this.originalImage || !this.currentCandidate) return;

        try {
            if (decision === "left") {
                // Keep Left (Original), Delete Right (Candidate)
                await this.deleteImage(this.currentCandidate.path);
            } else if (decision === "right") {
                // Keep Right (Candidate), Delete Left (Original)
                // BUT we need to keep one as the new "Original" for comparison against next?
                // Logic: "Winner stays on".
                await this.deleteImage(this.originalImage.path);
                this.originalImage = this.currentCandidate; // Candidate becomes the new king
            } else {
                // Keep both - do nothing, just advance
            }

            // Remove processed candidate from queue
            this.remainingImages.shift();
            this.showNextCandidate();

        } catch (e) {
            console.error("Feil ved resolving:", e);
            toast.show("Kunne ikke utføre handling: " + e, "error");
        }
    }

    private async deleteImage(path: string) {
        await invoke<OperationResult>("delete_images", { paths: [path] });
        // Also remove from the DOM/State in the background? 
        // For now we just delete the file.
        // We should dispatch an event to remove it from the main list.
        const event = new CustomEvent("image-deleted", { detail: { path } });
        document.dispatchEvent(event);
    }
}

export const comparisonManager = new ComparisonManager();
