// Copyright (C) 2025 aidan-es. Licensed under the GNU AGPLv3.
/**
 * Fetches the text content of a file from a given URL.
 * @param {string} url - The URL of the file to fetch.
 * @returns {Promise<string|null>} A promise that resolves with the text content of the file, or null if an error occurs.
 */
export async function fetch_text(url) {
    try {
        const response = await fetch(url);
        if (!response.ok) {
            console.error(`HTTP error! status: ${response.status} for ${url}`);
            return null;
        }
        return await response.text();
    } catch (e) {
        console.error(`Failed to fetch file at ${url}:`, e);
        return null;
    }
}

/**
 * Fetches the binary content (bytes) of an image file from a given URL.
 * @param {string} url - The URL of the image file to fetch.
 * @returns {Promise<Uint8Array|null>} A promise that resolves with the image data as a Uint8Array, or null if an error occurs.
 */
export async function fetch_image_bytes(url) {
    try {
        const response = await fetch(url);
        if (!response.ok) {
            console.error(`HTTP error! status: ${response.status} for ${url}`);
            return null;
        }
        const arrayBuffer = await response.arrayBuffer();
        return new Uint8Array(arrayBuffer);
    } catch (e) {
        console.error(`Failed to fetch image bytes at ${url}:`, e);
        return null;
    }
}

/**
 * Fetches the list of asset files from a manifest.
 * @param {string} url - The URL of the asset manifest JSON file.
 * @returns {Promise<string[]|null>} A promise that resolves with an array of file paths, or null if an error occurs.
 */
export async function fetch_asset_list(url) {
    try {
        const response = await fetch(url);
        if (!response.ok) {
            console.error(`HTTP error! status: ${response.status} for ${url}`);
            return null;
        }
        const data = await response.json();
        return data.files;
    } catch (e) {
        console.error(`Failed to fetch asset list at ${url}:`, e);
        return null;
    }
}

/**
 * Triggers a browser download for a file.
 * @param {Uint8Array} bytes - The file content as bytes.
 * @param {string} filename - The desired name for the downloaded file.
 */
export function trigger_download(bytes, filename) {
    // noinspection SpellCheckingInspection
    try {
        let mime_type = "application/octet-stream";
        if (filename.endsWith(".png")) {
            mime_type = "image/png";
        }

        const blob = new Blob([bytes], {type: mime_type});
        const url = URL.createObjectURL(blob);
        const a = document.createElement("a");

        a.href = url;
        a.download = filename;

        // Appending to the body is required for Firefox
        document.body.appendChild(a);
        a.click();

        document.body.removeChild(a);
        URL.revokeObjectURL(url);
    } catch (e) {
        console.error("Failed to trigger download:", e);
        throw e; // Re-throw to be caught by wasm_bindgen
    }
}
