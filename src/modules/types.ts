

export interface ImageInfo {
    path: string;
    filename: string;
    sizeBytes: number;
}

export interface ScanResult {
    imageCount: number;
    totalSizeBytes: number;
    images: ImageInfo[];
}

export interface DuplicateGroup {
    images: ImageInfo[];
}

export interface DuplicateResult {
    groups: DuplicateGroup[];
    totalDuplicates: number;
    processed: number;
    errors: number;
}

export interface OperationResult {
    processed: number;
    success: number;
    errors: number;
    errorMessages: string[];
}
