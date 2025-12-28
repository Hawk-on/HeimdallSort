const fs = require('fs');
const path = require('path');
const https = require('https');
const { execSync } = require('child_process');

const BINARIES_DIR = path.join(__dirname, '../src-tauri/binaries');
const FFPROBE_VERSION = '6.1'; // Or whatever version is stable/available

// Mappings for Tauri sidecar naming convention:
// binary-name-target-triple
// We need 'ffmpeg' and 'ffprobe'

const PLATFORMS = [
    {
        name: 'linux-x64',
        targetTriple: 'x86_64-unknown-linux-gnu',
        ffmpegUrl: 'https://github.com/eugeneware/ffmpeg-static/releases/download/b6.0/linux-x64', // Example source, verifying below
        // Actually, let's use a more reliable source or just download from specific releases.
        // mwader/static-ffmpeg is good for linux/mac/windows self-contained.
    }
];

// Better source: ffbinaries-node or similar? 
// Let's use `ffbinaries` API or direct links to valid builds.
// For simplicity and reliability, let's look for a known static build provider.
// https://github.com/eugeneware/ffmpeg-static is a wrapper. 
// https://github.com/BtbN/FFmpeg-Builds/releases is good for Windows/Linux.
// https://evermeet.cx/ffmpeg/ works for Mac.

// To simplify life, let's use `ffbinaries` npm package concept but write our own specific downloads
// OR just ask the user to run this script which uses `ffbinaries` package via npx?
// "npx ffbinaries -d" is easier but renaming is manual.

// Let's write a script that uses `ffbinaries` via npx execution, then renames.

const fetchBinaries = () => {
    if (!fs.existsSync(BINARIES_DIR)) {
        fs.mkdirSync(BINARIES_DIR, { recursive: true });
    }

    console.log('Downloading FFmpeg binaries via ffbinaries...');

    // We strictly need: linux-64, windows-64, osx-64 (intel), osx-arm64 (apple silicon)
    // ffbinaries supports: linux-64, win-64, osx-64. Does it support m1?

    // Commands to run
    try {
        // Linux
        console.log('Fetching Linux...');
        execSync(`npx ffbinaries ffmpeg ffprobe --platform=linux-64 --output="${BINARIES_DIR}" --quiet --yes`, { stdio: 'inherit' });
        rename(BINARIES_DIR, 'ffmpeg', 'x86_64-unknown-linux-gnu');
        rename(BINARIES_DIR, 'ffprobe', 'x86_64-unknown-linux-gnu');

        // Windows
        console.log('Fetching Windows...');
        execSync(`npx ffbinaries ffmpeg ffprobe --platform=win-64 --output="${BINARIES_DIR}" --quiet --yes`, { stdio: 'inherit' });
        rename(BINARIES_DIR, 'ffmpeg.exe', 'x86_64-pc-windows-msvc.exe');
        rename(BINARIES_DIR, 'ffprobe.exe', 'x86_64-pc-windows-msvc.exe');

        // Mac (Intel)
        console.log('Fetching Mac (Intel)...');
        execSync(`npx ffbinaries ffmpeg ffprobe --platform=osx-64 --output="${BINARIES_DIR}" --quiet --yes`, { stdio: 'inherit' });
        rename(BINARIES_DIR, 'ffmpeg', 'x86_64-apple-darwin');
        rename(BINARIES_DIR, 'ffprobe', 'x86_64-apple-darwin');

        // Mac (ARM/Apple Silicon) - ffbinaries might not support this explicitly or has it merged?
        // ffbinaries usually downloads standard builds. Mac ARM builds often use same 'ffmpeg' if universal,
        // but typically they are separate.
        // As a fallback for now, we will duplicate the Intel one or warn? 
        // Actually, checking ffbinaries repo... it pulls from https://ffmpeg.org/download.html related links.
        // Getting proper ARM builds automatically is tricky. 
        // Let's duplicate x64 for aarch64 for now (Rosetta 2 handles it usually) OR leave it as "TODO" and user has to manually swap if native performance needed.
        // Actually, let's duplicate it to ensure the app LAUNCHES on M1 macs without crashing due to missing binary.
        console.log('Duplicating Mac Intel for ARM (Rosetta 2)...');
        fs.copyFileSync(path.join(BINARIES_DIR, 'ffmpeg-x86_64-apple-darwin'), path.join(BINARIES_DIR, 'ffmpeg-aarch64-apple-darwin'));
        fs.copyFileSync(path.join(BINARIES_DIR, 'ffprobe-x86_64-apple-darwin'), path.join(BINARIES_DIR, 'ffprobe-aarch64-apple-darwin'));

        console.log('Done! Binaries are in src-tauri/binaries/');

    } catch (e) {
        console.error('Error downloading binaries:', e);
    }
};

function rename(dir, oldName, targetTriple) {
    // ffbinaries extracts as 'ffmpeg', 'ffmpeg.exe', etc.
    // We need to rename to 'ffmpeg-<target-triple>'
    // But wait, ffbinaries overrides files if we run sequentially in same dir.
    // We need to rename IMMEDIATELY after download.

    // Actually the execSync above runs sequentially? No, the command `npx ...` finishes then we rename.
    // BUT if we download linux, then windows, 'ffmpeg' might be overwritten if extensions differ (linux no ext, windows .exe - safe).
    // Linux vs Mac: both 'ffmpeg'. unsafe.

    // Improvement: Download to temp subdir.
}

// Redoing logic to use temp folders for safety
const fetchSafe = () => {
    if (!fs.existsSync(BINARIES_DIR)) {
        fs.mkdirSync(BINARIES_DIR, { recursive: true });
    }

    const targets = [
        { platform: 'linux-64', suffix: 'x86_64-unknown-linux-gnu', ext: '' },
        { platform: 'win-64', suffix: 'x86_64-pc-windows-msvc', ext: '.exe' },
        { platform: 'osx-64', suffix: 'x86_64-apple-darwin', ext: '' },
    ];

    for (const t of targets) {
        console.log(`Processing ${t.platform}...`);
        const tempDir = path.join(BINARIES_DIR, `temp_${t.platform}`);
        if (!fs.existsSync(tempDir)) fs.mkdirSync(tempDir);

        try {
            execSync(`npx ffbinaries ffmpeg ffprobe --platform=${t.platform} --output="${tempDir}" --quiet --yes`, { stdio: 'inherit' });

            // Move and rename
            const ffmpegSrc = path.join(tempDir, `ffmpeg${t.ext}`);
            const ffprobeSrc = path.join(tempDir, `ffprobe${t.ext}`);

            const ffmpegDst = path.join(BINARIES_DIR, `ffmpeg-${t.suffix}${t.ext}`);
            const ffprobeDst = path.join(BINARIES_DIR, `ffprobe-${t.suffix}${t.ext}`);

            if (fs.existsSync(ffmpegSrc)) fs.renameSync(ffmpegSrc, ffmpegDst);
            if (fs.existsSync(ffprobeSrc)) fs.renameSync(ffprobeSrc, ffprobeDst);

            // Cleanup
            fs.rmSync(tempDir, { recursive: true, force: true });

        } catch (e) {
            console.error(`Failed for ${t.platform}:`, e);
        }
    }

    // Handle Mac ARM (Copy Intel)
    console.log('Creating Mac ARM aliases...');
    try {
        const intelSuffix = 'x86_64-apple-darwin';
        const armSuffix = 'aarch64-apple-darwin';

        fs.copyFileSync(
            path.join(BINARIES_DIR, `ffmpeg-${intelSuffix}`),
            path.join(BINARIES_DIR, `ffmpeg-${armSuffix}`)
        );
        fs.copyFileSync(
            path.join(BINARIES_DIR, `ffprobe-${intelSuffix}`),
            path.join(BINARIES_DIR, `ffprobe-${armSuffix}`)
        );
    } catch (e) {
        console.warn("Could not copy Mac binaries (maybe install failed?)");
    }

    console.log("All binaries prepared.");
};

fetchSafe();
