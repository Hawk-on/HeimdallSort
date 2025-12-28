import { state } from "./state";
import { ImageInfo } from "./types";

let renderCallback: ((img: ImageInfo, index: number) => HTMLElement) | null = null;
let containerRef: HTMLElement | null = null;
let spacerRef: HTMLElement | null = null;
let contentRef: HTMLElement | null = null;

export function setupVirtualScroll(
    container: HTMLElement,
    spacer: HTMLElement,
    content: HTMLElement,
    renderer: (img: ImageInfo, index: number) => HTMLElement
) {
    containerRef = container;
    spacerRef = spacer;
    contentRef = content;
    renderCallback = renderer;

    const calculateMetrics = () => {
        if (!containerRef) return;
        const containerWidth = containerRef.clientWidth;
        // Min width 160px + gap 16px
        const minColWidth = 160 + 16;
        state.virtualState.cols = Math.max(1, Math.floor((containerWidth - 32) / minColWidth));
        state.virtualState.totalRows = Math.ceil(state.currentImages.length / state.virtualState.cols);

        // Sync CSS grid columns
        contentRef?.style.setProperty('--grid-cols', String(state.virtualState.cols));

        const gap = 16;
        const padding = 16;
        const availableWidth = containerWidth - padding;
        const colWidth = (availableWidth - (state.virtualState.cols - 1) * gap) / state.virtualState.cols;

        // Height = colWidth (aspect-1) + info (50) + gap
        state.virtualState.rowHeight = colWidth + 50 + gap;

        // Update spacer height
        const totalHeight = state.virtualState.totalRows * state.virtualState.rowHeight;
        if (spacerRef) spacerRef.style.height = `${totalHeight}px`;

        state.virtualState.containerHeight = containerRef.clientHeight;
    };

    // Resize Observer
    const resizeObserver = new ResizeObserver(() => {
        calculateMetrics();
        renderVirtualItems();
    });
    resizeObserver.observe(containerRef);

    // Scroll listener
    containerRef.addEventListener("scroll", (e) => {
        requestAnimationFrame(() => {
            state.virtualState.scrollTop = (e.target as HTMLElement).scrollTop;
            renderVirtualItems();
        });
    });

    // Initial calculation
    calculateMetrics();
    renderVirtualItems();
}

export function renderVirtualItems() {
    if (!contentRef || !state.currentImages.length || !renderCallback) return;

    // Calculate visible rows
    const startRow = Math.floor(state.virtualState.scrollTop / state.virtualState.rowHeight);
    const visibleRows = Math.ceil(state.virtualState.containerHeight / state.virtualState.rowHeight);

    // Buffer
    const buffer = 2;
    const startRowWithBuffer = Math.max(0, startRow - buffer);
    let endRowWithBuffer = startRow + visibleRows + buffer;
    endRowWithBuffer = Math.min(endRowWithBuffer, state.virtualState.totalRows);

    const newStartIndex = startRowWithBuffer * state.virtualState.cols;
    const newEndIndex = Math.min(endRowWithBuffer * state.virtualState.cols, state.currentImages.length);

    if (newStartIndex === state.virtualState.startIndex && newEndIndex === state.virtualState.endIndex) {
        return;
    }

    state.virtualState.startIndex = newStartIndex;
    state.virtualState.endIndex = newEndIndex;

    // Position content div
    const offsetY = startRowWithBuffer * state.virtualState.rowHeight;
    contentRef.style.transform = `translateY(${offsetY}px)`;

    // Clear and refill
    contentRef.innerHTML = "";

    const visibleImages = state.currentImages.slice(newStartIndex, newEndIndex);
    const fragment = document.createDocumentFragment();

    visibleImages.forEach((img, i) => {
        const actualIndex = newStartIndex + i;
        if (renderCallback) {
            fragment.appendChild(renderCallback(img, actualIndex));
        }
    });
    contentRef.appendChild(fragment);

    // Update header count if exists
    const headerTitle = document.querySelector("#gallery-header h2");
    if (headerTitle) {
        headerTitle.textContent = `📷 Bilder (${state.currentImages.length} totalt)`;
    }
}
