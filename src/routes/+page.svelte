<script lang="ts">
	// import { invoke } from "@tauri-apps/api/core";
	import { listen } from "@tauri-apps/api/event";
	import Entry from "../lib/components/Entry.svelte";
	import { cbEventType } from "../lib/types";
	import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";

	let cbLog: Array<string> = $state([]);

	listen<cbEventType>("cb-text-copy", async (type) => {
		console.log(`event received`);
		const text = await readText();
		console.log(text);
		cbLog.unshift(text.trim());
	});
</script>

<main class="container">
	<div>
		<p>Squirrel</p>
	</div>
	<div>
		<p><small>Clipboard history:</small></p>
		{#each cbLog as text}
			<Entry content={text} type={cbEventType.Text} />
		{/each}
	</div>
</main>

<style>
	:root {
		font-family: sans-serif;
		font-size: 16px;

		color: #0f0f0f;
		background-color: #f6f6f6;

		font-synthesis: none;
		text-rendering: optimizeLegibility;
		-webkit-font-smoothing: antialiased;
		-moz-osx-font-smoothing: grayscale;
		-webkit-text-size-adjust: 100%;
	}

	.container {
		margin: 0;
		display: flex;
		flex-direction: column;
		padding: 0rem 2rem;
	}

	p {
		color: #555;
	}

	div {
		display: flex;
		flex-direction: column;
	}

	/* @media (prefers-color-scheme: dark) {
		:root {
			color: #f6f6f6;
			background-color: #2f2f2f;
		}
	} */
</style>
