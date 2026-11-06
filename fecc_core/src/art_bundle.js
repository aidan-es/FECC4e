// Copyright (C) 2026 aidan-es. Licensed under the GNU AGPLv3.
// Fetches and reads art bundles, for the app and the 404 page. See the fecc_art_bundle crate.

/**
 * Fetches a gzip file and decompresses it.
 * The art bundles are compressed ahead of time, as Cloudflare doesn't compress binary files.
 * @param {string} url - The URL of the file to fetch.
 * @param {string} priority - The fetch priority hint: "auto" or "low" (for background preloading).
 * @returns {Promise<Uint8Array>} A promise that resolves with the decompressed bytes.
 * @throws {Error} If the fetch fails or the file isn't valid gzip.
 */
export async function fetch_gzip_bytes(url, priority) {
    const response = await fetch(url, { priority });
    if (!response.ok) {
        throw new Error(`HTTP status ${response.status}`);
    }
    // Read it all first, as decompressing it as it arrives cancels the request at the end of
    // the gzip data, which can keep it out of the HTTP cache
    const compressed = await response.blob();
    const decompressed = compressed.stream().pipeThrough(new DecompressionStream("gzip"));
    return new Uint8Array(await new Response(decompressed).arrayBuffer());
}

/**
 * Lists the files in an art bundle.
 * A bundle is "FECCART1", then for each file: name length (u16 LE), UTF-8 name,
 * data length (u32 LE), data.
 * @param {Uint8Array} bundle - The decompressed bundle.
 * @returns {{name: string, data: Uint8Array}[]} Each file, in order.
 * @throws {Error} If it isn't an art bundle or is cut short.
 */
export function read_art_bundle(bundle) {
    const magic = new TextEncoder().encode("FECCART1");
    if (bundle.length < magic.length || magic.some((byte, i) => bundle[i] !== byte)) {
        throw new Error("Not an art bundle");
    }

    const view = new DataView(bundle.buffer, bundle.byteOffset, bundle.byteLength);
    const names = new TextDecoder("utf-8", { fatal: true });
    const take = (length) => {
        if (offset + length > bundle.length) {
            throw new Error("The art bundle is cut short");
        }
        offset += length;
        return offset - length;
    };

    const files = [];
    let offset = magic.length;
    while (offset < bundle.length) {
        const nameLength = view.getUint16(take(2), true);
        const nameStart = take(nameLength);
        const name = names.decode(bundle.subarray(nameStart, nameStart + nameLength));
        const dataLength = view.getUint32(take(4), true);
        const dataStart = take(dataLength);
        files.push({ name, data: bundle.subarray(dataStart, dataStart + dataLength) });
    }
    return files;
}
