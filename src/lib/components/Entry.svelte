<script lang="ts">
	import type { clipboardEvent } from "$lib/types";
	import { invoke } from "@tauri-apps/api/core";
	import { imageCache } from "$lib/cache";

	const {
		cbEvent,
		active = false,
		onClick,
		onDelete,
		onPin,
	}: {
		cbEvent: clipboardEvent;
		active?: boolean;
		onClick: () => void;
		onDelete: (id: number) => void;
		onPin: (id: number) => void;
	} = $props();
	import { getCurrentWindow } from "@tauri-apps/api/window";

	const window = getCurrentWindow();
	export const id = () => {
		return cbEvent.id;
	};

	// Whether to show the checkmark or the copy icon
	let isCopied = $state(false);
	// Whether this entry is pinned
	let isPinned = $derived(cbEvent.is_pinned);
	// This element
	let element: HTMLElement | null = $state(null);
	// Wrapper around this entry's content
	let contentWrapperElement: HTMLDivElement | null = $state(null);
	// If the entry is of the image type
	let imageData: string | null = $state(null);

	// Lazy loading for image data
	$effect(() => {
		if (cbEvent.event_type !== "image" || !contentWrapperElement) return;
		const observer = new IntersectionObserver(async ([e]) => {
			if (e.isIntersecting && !imageData) {
				imageData = await imageCache.get(cbEvent.id);
				observer.disconnect();
			}
		});

		observer.observe(contentWrapperElement);
		return () => observer.disconnect();
	});

	/**
	 * Focuses this element.
	 */
	export function focusElement() {
		// non-null assertion: Element always exists
		element!.focus();
	}

	/**
	 * Invokes the `copy_item` command in the backend, copying the Entry's
	 * content to the clipboard.
	 */
	export async function handleCopy() {
		isCopied = true;
		invoke("copy_item", { id: cbEvent.id });

		setTimeout(() => {
			isCopied = false;
		}, 1000);
	}

	/**
	 * Sets the pinned state of the Entry and invokes the `pin_entry` command
	 * in the backend. Also calls the UI pin function.
	 */
	export function handlePin() {
		invoke("pin_entry", { id: cbEvent.id, isPinned });
		onPin(cbEvent.id);
		if (isPinned) {
			onClick();
		}
	}

	/**
	 * Invokes the paste_item command in the backend, pasting the Entry's
	 * content via Ctrl+V emulation.
	 */
	export function handlePaste() {
		invoke("paste_item", { id: cbEvent.id });
	}

	/**
	 * Invokes the remove_entry command in the backend, deleting this entry
	 * from the database and calling the UI removal function.
	 */
	export async function handleRemove() {
		invoke("remove_entry", { id: cbEvent.id });
		onDelete(cbEvent.id);
	}

	function handleKeyDown(event: KeyboardEvent) {
		if (event.ctrlKey || event.metaKey) {
			switch (event.key) {
				case "c": {
					handleCopy();
					break;
				}
				case "p": {
					handlePin();
					break;
				}
			}
		} else {
			switch (event.key) {
				case "Enter": {
					window.hide();
					setTimeout(() => {
						handlePaste();
					}, 5);
					break;
				}
				case "Delete": {
					handleRemove();
					break;
				}
			}
		}
	}
</script>

<div
	class="entry-container list-item {isPinned ? 'pinned' : ''}"
	onkeydown={handleKeyDown}
	bind:this={element}
	class:active
	tabindex="-1"
	role="option"
	onclick={onClick}
	aria-selected={active}>
	<div class="content" bind:this={contentWrapperElement}>
		{#if cbEvent.content.type == "Text"}
			{#if cbEvent.content.data}
				<p data-selectable="true">{cbEvent.content.data}</p>
			{:else}
				<span class="content-loading-text">loading content...</span>
			{/if}
		{:else if cbEvent.content.type == "Image"}
			<div bind:this={contentWrapperElement} style="min-height: 2rem">
				{#if imageData}
					<img
						style="max-width: 100%; max-height: 8rem"
						src={`data:image/png;base64,${imageData}`}
						alt="clipboard" />
				{:else}
					<span class="content-loading-text">loading image...</span>
				{/if}
			</div>
		{/if}
	</div>
	<div class="footer">
		<span class="time">{new Date(cbEvent.timestamp).toLocaleString()}</span>
		<div class="buttons">
			<!-- Icons from https://lucide.dev -->
			<button onclick={handleCopy} title="Copy">
				{#if isCopied}
					<svg
						xmlns="http://www.w3.org/2000/svg"
						width="24"
						height="24"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						color="green"
						stroke-width="2"
						stroke-linecap="round"
						stroke-linejoin="round"
						class="check-icon"
						><path d="M20 6 9 17l-5-5" />
					</svg>
				{:else}
					<svg
						xmlns="http://www.w3.org/2000/svg"
						width="24"
						height="24"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
						stroke-linecap="round"
						stroke-linejoin="round"
						class="copy-icon">
						<rect width="14" height="14" x="8" y="8" rx="2" ry="2" /><path
							d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2" />
					</svg>
				{/if}
			</button>
			<button
				onclick={() => {
					handlePin();
				}}
				title={isPinned ? "Unpin" : "Pin"}>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					width="24"
					height="24"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
					class="pin-icon {isPinned ? 'filled' : ''}">
					<path d="M12 17v5" /><path
						d="M9 10.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24V16a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1v-.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V7a1 1 0 0 1 1-1 2 2 0 0 0 0-4H8a2 2 0 0 0 0 4 1 1 0 0 1 1 1z" />
				</svg>
			</button>
			<button onclick={handleRemove} title="Delete">
				<svg
					xmlns="http://www.w3.org/2000/svg"
					width="24"
					height="24"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
					class="cross-icon">
					<path d="M18 6 6 18" /><path d="m6 6 12 12" />
				</svg>
			</button>
		</div>
	</div>
</div>

<style>
	.entry-container {
		box-sizing: border-box;
		min-width: 0;
		max-width: 100%;
		font-size: 0.8rem;
		background-color: var(--bg-secondary);
		border: 1px solid var(--bg-accent);
		border-radius: 3px;
		padding: 0.5rem;
		display: flex;
		flex-direction: column;
		justify-content: space-between;
		margin-bottom: 0.2rem;
	}

	.entry-container:focus {
		z-index: 9999;
		background-color: var(--entry-focus);
		outline: 1px solid var(--black);
	}

	.pinned {
		background-color: var(--entry-pinned);
	}

	.pinned:focus {
		background-color: var(--entry-pinned-focus);
	}

	.content {
		scrollbar-gutter: stable;
		padding-right: 0.5rem;
		min-width: 0;
		width: 100%;
		flex-grow: 1;
	}

	.content-loading-text {
		font-size: 0.75rem;
		color: var(--bg-accent);
	}

	.footer {
		margin-top: 0.5rem;
		height: 1rem;
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.time {
		font-family: monospace;
		font-size: 0.6rem;
		color: var(--fg-accent);
	}

	p {
		margin-bottom: 0;
		margin-top: 0.25rem;
		max-height: 6rem;
		mask-image: linear-gradient(to bottom, black calc(100% - 0.2rem), transparent 100%);
		max-width: 100%;
		user-select: text;
		overflow-y: auto;
		overflow-wrap: break-word;
		white-space: pre-wrap;
	}

	::-webkit-scrollbar {
		width: 3px;
	}

	::-webkit-scrollbar-thumb {
		border-radius: 50px;
		background: var(--bg-accent);
	}

	::-webkit-scrollbar-thumb:hover {
		background-color: var(--bg-secondary);
	}

	.buttons {
		display: flex;
	}

	button {
		all: unset;
		display: flex;
		align-items: center;
		justify-content: center;
		height: 1rem;
		width: 1rem;
		margin-left: 0.2rem;
		cursor: pointer;
		text-align: center;
		border-radius: 100px;
		transition: all 200ms;
	}

	svg {
		height: 0.75rem;
		color: var(--fg-accent);
	}

	button:active {
		background-color: var(--bg-secondary);
	}

	svg:hover {
		color: var(--black);
	}

	.cross-icon:hover {
		color: red;
	}

	.filled {
		fill: var(--fg-primary);
		color: var(--fg-primary);
	}

	.check-icon,
	.check-icon:hover {
		color: green;
	}
</style>
