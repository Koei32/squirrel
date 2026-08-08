import { invoke } from "@tauri-apps/api/core";

/**
 * A simple image cache.
 */
class ImageCache {
	private cache: Map<number, string> = new Map();
	private max_items: number = 32; // arbitrary

	/**
	 * Get the content of an entry from the cache if it exists, or requested from the backend,
	 * inserting into the cache.
	 * @param id The id of the entry whose data is to be returned
	 * @returns Base64 image data
	 */
	public async get(id: number): Promise<string> {
		if (this.cache.has(id)) {
			return this.cache.get(id)!;
		} else {
			const data: string = await invoke("get_entry_content", { id: id });
			if (this.cache.size >= this.max_items) {
				this.cache.delete(this.cache.keys().next().value!);
			}
			this.cache.set(id, data);
			return data;
		}
	}
}

export const imageCache = new ImageCache();
