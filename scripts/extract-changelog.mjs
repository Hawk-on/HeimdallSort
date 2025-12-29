import { readFileSync } from 'fs';
import { resolve } from 'path';

try {
    const changelogPath = resolve('CHANGELOG.md');
    const content = readFileSync(changelogPath, 'utf-8');
    const lines = content.split(/\r?\n/);

    let capture = false;
    let buffer = [];

    for (const line of lines) {
        // Match version header, e.g. "## [1.2.0] - 2024-..."
        if (line.match(/^## \[\d+\.\d+\.\d+\]/)) {
            if (capture) {
                // If we were already capturing, this means we hit the NEXT version -> STOP
                break;
            }
            // First time seeing a version -> START capturing
            capture = true;
            continue;
        }

        if (capture) {
            buffer.push(line);
        }
    }

    if (buffer.length > 0) {
        console.log(buffer.join('\n').trim());
    } else {
        console.error('Could not find latest version in CHANGELOG.md');
        process.exit(1);
    }

} catch (error) {
    console.error('Error processing CHANGELOG.md:', error);
    process.exit(1);
}
