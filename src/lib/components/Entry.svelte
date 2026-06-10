<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";
	const { cbEvent, active = false, clickHandler, onCopy, onPin, onDelete } = $props();

	// let isCopied = $state(false);
	let element: HTMLElement | null = $state(null);

	export function focusElement() {
		if (element) {
			element.focus();
		}
	}

	export function handlePaste() {
		invoke("paste_item", { id: cbEvent.id });
	}

	export async function handleRemove() {
		invoke("remove_entry", { id: cbEvent.id });
		await onDelete();
	}

	export function handleCopy() {
		invoke("copy_item", { id: cbEvent.id });
	}
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
	class="entry-container list-item"
	bind:this={element}
	class:active
	tabindex="-1"
	role="option"
	aria-selected={active}
	onclick={clickHandler}>
	<div class="content">
		<p data-selectable="true">
			{cbEvent.content}
		</p>
	</div>
	<div class="footer">
		<span class="time">{new Date(cbEvent.timestamp).toLocaleString()}</span>
		<div class="buttons">
			<button onclick={onCopy} title="Copy">
				<svg
					xmlns="http://www.w3.org/2000/svg"
					width="24"
					height="24"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round">
					<rect width="14" height="14" x="8" y="8" rx="2" ry="2" /><path
						d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2" />
				</svg>
			</button>
			<button onclick={onPin} title="Pin">
				<svg
					xmlns="http://www.w3.org/2000/svg"
					width="24"
					height="24"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round">
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
					stroke-linejoin="round">
					<path d="M18 6 6 18" /><path d="m6 6 12 12" />
				</svg>
			</button>
		</div>
	</div>
</div>

<style>
	.entry-container {
		min-width: 0;
		max-width: 100%;
		font-size: 0.8rem;
		background-color: #eee;
		border: 1px solid #bbb;
		border-radius: 3px;
		padding: 0.5rem;
		display: flex;
		flex-direction: column;
		justify-content: space-between;
		/* align-items: center; */
		margin-bottom: 0.1rem;
	}

	.entry-container:active {
		background-color: #ddd;
	}

	.entry-container:focus {
		background-color: #ddd;
		outline: 1px solid grey;
	}

	.content {
		scrollbar-gutter: stable;
		padding-right: 0.5rem;
		min-width: 0;
		width: 100%;
		flex-grow: 1;
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
		color: #aaa;
	}

	p {
		margin-bottom: 0;
		margin-top: 0.25rem;
		max-height: 6rem;
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
		background: #ccc;
	}

	::-webkit-scrollbar-thumb:hover {
		background: #bbb;
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
		/* outline: none; */
	}

	svg {
		height: 0.75rem;
		/* fill: #bbb; */
		color: #aaa;
	}

	/* button:hover {
		transition: all 200ms;
		box-shadow: 0px 0px 3px rgb(192, 192, 192);
		background-color: #999;
	} */

	button:hover > svg {
		transition: all 200ms;
		color: black;
	}

	button:active {
		background-color: #eee;
	}
</style>
