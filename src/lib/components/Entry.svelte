<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";

	const { type, content, timestamp, active = false, clickHandler } = $props();

	let isCopied = $state(false);
	let element: HTMLElement | null = $state(null);

	export function focusElement() {
		if (element) {
			element.focus();
		}
	}

	export function handlePaste() {
		invoke("paste_content", { content: content });
	}

	export function handleCopy() {
		isCopied = true;
		invoke("copy_content");

		setTimeout(() => {
			isCopied = false;
		}, 2000);
	}
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="entry-container list-item"
	bind:this={element}
	class:active
	tabindex="-1"
	role="option"
	aria-selected={active}
	onclick={clickHandler}>
	<div class="content">
		<span class="time">{new Date(timestamp).toLocaleString()}</span>
		<p data-selectable="true">
			{content}
		</p>
	</div>
	<button onclick={handleCopy}>
		{#if isCopied}
			✓
		{:else}
			Copy
		{/if}
	</button>
</div>

<style>
	.entry-container {
		font-size: 0.8rem;
		background-color: #eee;
		border: 1px solid #bbb;
		border-radius: 3px;
		padding: 0.75rem;
		display: flex;
		justify-content: space-between;
		align-items: center;
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
		display: flex;
		width: 80%;
		flex-direction: column;
	}

	.time {
		font-family: monospace;
		font-size: 0.6rem;
		color: #bbb;
	}

	p {
		margin-bottom: 0;
		margin-top: 0.25rem;
		max-height: 12rem;
		user-select: text;
		overflow-y: auto;
		overflow-x: auto;
		overflow-wrap: break-word;
		white-space: pre-wrap;
	}

	button {
		all: unset;
		padding: 0.5rem;
		height: 1rem;
		cursor: pointer;
		background-color: white;
		min-width: 2rem;
		text-align: center;
		border-radius: 100px;
		transition: all 200ms;
		outline: none;
	}

	button:hover {
		transition: all 200ms;
		box-shadow: 0px 1px 3px rgb(192, 192, 192);
	}

	button:active {
		background-color: #eee;
	}
</style>
