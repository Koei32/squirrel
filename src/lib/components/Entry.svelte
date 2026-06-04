<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";

	const { type, content, timestamp } = $props();

	let isCopied = $state(false);
	let textSpan: HTMLParagraphElement | null = $state(null);

	function handleCopyClick() {
		isCopied = true;
		invoke("copy_content");

		setTimeout(() => {
			isCopied = false;
		}, 2000);
	}

	function focusOnContent(event: MouseEvent) {
		const target = event.target as HTMLElement;
		if (textSpan) {
			const selection = window.getSelection();
			const range = document.createRange();

			if (selection) {
				range.selectNodeContents(textSpan);
				selection.removeAllRanges();
				selection.addRange(range);
			}
		}
	}
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="container">
	<div class="content">
		<span class="type">{new Date(timestamp).toLocaleString()}</span>
		<p bind:this={textSpan} data-selectable="true">
			{content}
		</p>
	</div>
	<button onclick={handleCopyClick}>
		{#if isCopied}
			✓
		{:else}
			Copy
		{/if}
	</button>
</div>

<style>
	.content {
		display: flex;
		width: 80%;
		flex-direction: column;
	}

	.container {
		font-size: 0.8rem;
		background-color: #eee;
		border: 1px solid #bbb;
		border-radius: 10px;
		/* height: 2.5rem; */
		padding: 0.75rem;
		/* width: 80%; */
		display: flex;
		/* flex-direction: column; */
		justify-content: space-between;
		align-items: center;
		margin-bottom: 0.31rem;
	}

	.type {
		font-family: monospace;
		font-size: 0.6rem;
		color: #555;
	}

	p {
		margin-bottom: 0;
		max-height: 12rem;
		user-select: text;
		overflow-y: auto;
		overflow-x: auto;
		overflow-wrap: break-word;
		white-space: pre-wrap;
	}

	button {
		all: unset;
		/* font-size: 0.75rem; */
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
