<script lang="ts">
	import { listen } from "@tauri-apps/api/event";
	import Entry from "../lib/components/Entry.svelte";
	import type { cbEventNotice, clipboardEvent } from "../lib/types";
	import { cbEventType } from "../lib/types";
	import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";
	import { invoke } from "@tauri-apps/api/core";

	let cbLog: Array<clipboardEvent> = $state([]);
	let count = $state(0);

	listen<cbEventNotice>("cb-text-copy", async (event) => {
		console.log(`${event.payload.event_type} event received at ${event.payload.timestamp}`);
		// count++;
		const text = await readText();
		cbLog.unshift({
			event_type: event.payload.event_type,
			content: text,
			timestamp: event.payload.timestamp,
		});
	});

	async function clearHistory() {
		await invoke("clear_history").catch((r) => {
			console.log(r);
		});
		cbLog = [];
	}
</script>

<main class="container">
	<div>
		<p>Squirrel {count}</p>
	</div>
	<div>
		<div class="subhead">
			<p><small>Clipboard history:</small></p>
			<button onclick={clearHistory}>Clear History (permanent!)</button>
		</div>
		{#each cbLog as event}
			<Entry content={event.content} type={event.event_type} timestamp={event.timestamp} />
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

	.subhead {
		flex-direction: row;
		justify-content: space-between;
		align-items: center;
	}

	button {
		all: unset;
		font-size: 0.75rem;
		padding: 0.5rem;
		height: 1rem;
		cursor: pointer;
		background-color: rgb(255, 164, 164);
		color: maroon;
		font-weight: bold;
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

	/* @media (prefers-color-scheme: dark) {
		:root {
			color: #f6f6f6;
			background-color: #2f2f2f;
		}
	} */
</style>
