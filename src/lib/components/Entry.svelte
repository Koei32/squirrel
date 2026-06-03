<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";
	import { writeText } from "@tauri-apps/plugin-clipboard-manager";

	const { content } = $props();

	let isCopied = $state(false);
	let textSpan: HTMLSpanElement | null = $state(null);

	function handleCopyClick() {
		isCopied = true;
		invoke("copy_content", content);

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
<div class="container" onclick={focusOnContent} data-selectable="true">
	<span bind:this={textSpan}>
		{content}
	</span>
	<button onclick={handleCopyClick}>
		{#if isCopied}
			✓
		{:else}
			Copy
		{/if}
	</button>
</div>

<style>
	div {
		font-size: 0.75rem;
		background-color: #eee;
		border-radius: 10px;
		/* height: 2.5rem; */
		padding: 0.5rem 0.75rem;
		/* width: 80%; */
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 0.31rem;
	}

	span {
		/* background-color: #fff; */
		/* height: 1rem; */
		/* width: 100%;
		padding: 0.5rem;
		margin-right: 1rem;
		border-radius: 5px; */
		user-select: all;
		white-space: nowrap;
		text-overflow: ellipsis;
		overflow: hidden;
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
</style>
