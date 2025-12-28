import { ImageInfo } from "./types";

export const CONFIG = {
    DUPLICATE_THRESHOLD: 5
};

export interface VirtualScrollState {
    rowHeight: number;
    containerHeight: number;
    scrollTop: number;
    cols: number;
    totalRows: number;
    startIndex: number;
    endIndex: number;
}

class AppState {
    currentImages: ImageInfo[] = [];
    selectedPaths: Set<string> = new Set();
    thumbnailCache: Map<string, string> = new Map();

    virtualState: VirtualScrollState = {
        rowHeight: 200,
        containerHeight: 0,
        scrollTop: 0,
        cols: 4,
        totalRows: 0,
        startIndex: 0,
        endIndex: 0
    };

    setImages(images: ImageInfo[]) {
        this.currentImages = images;
    }

    clearSelection() {
        this.selectedPaths.clear();
    }

    addToSelection(path: string) {
        this.selectedPaths.add(path);
    }

    removeFromSelection(path: string) {
        this.selectedPaths.delete(path);
    }

    toggleSelection(path: string) {
        if (this.selectedPaths.has(path)) {
            this.selectedPaths.delete(path);
            return false;
        } else {
            this.selectedPaths.add(path);
            return true;
        }
    }

    getSelectedImages(): ImageInfo[] {
        return this.currentImages.filter(img => this.selectedPaths.has(img.path));
    }
}

export const state = new AppState();
