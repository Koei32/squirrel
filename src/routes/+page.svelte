<script lang="ts">
	import { listen, type UnlistenFn } from "@tauri-apps/api/event";
	import Entry from "../lib/components/Entry.svelte";
	import type { cbEventNotice, clipboardEvent } from "../lib/types";
	import { readText } from "@tauri-apps/plugin-clipboard-manager";
	import { invoke } from "@tauri-apps/api/core";
	import { getCurrentWindow } from "@tauri-apps/api/window";
	import { onMount } from "svelte";
	import { TrayIcon, type TrayIconOptions } from "@tauri-apps/api/tray";
	import { Menu } from "@tauri-apps/api/menu";
	import { register } from "@tauri-apps/plugin-global-shortcut";
	import { exit } from "@tauri-apps/plugin-process";

	const window = getCurrentWindow();

	let cbLog: Array<clipboardEvent> = $state([]);
	let entries: Array<Entry> = $state([]);
	let activeIndex = $state(0);

	listen<cbEventNotice>("cb-text-copy", async (event) => {
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
		entries = [];
	}

	function handleNavigation(event: KeyboardEvent) {
		if (event.key == "Escape") {
			event.preventDefault();
			window.hide();
			return;
		}
		if (entries.length === 0) return;

		switch (event.key) {
			case "ArrowDown": {
				event.preventDefault();
				activeIndex = (activeIndex + 1) % cbLog.length;
				focusEntry();
				break;
			}
			case "ArrowUp": {
				event.preventDefault();
				activeIndex = (activeIndex - 1 + cbLog.length) % cbLog.length;
				focusEntry();
				break;
			}
			case "Enter": {
				window.hide();
				setTimeout(() => {
					entries[activeIndex].handlePaste();
				}, 50);
				break;
			}
		}
	}

	function focusEntry() {
		const targetComponent = entries[activeIndex];
		if (targetComponent) {
			targetComponent.focusElement();
		}
	}

	onMount(() => {
		let unlisten: UnlistenFn | undefined;

		const initSetup = async () => {
			// register shortcut to show the window
			await register("CommandOrControl+Shift+V", (event) => {
				if (event.state == "Pressed") {
					window.show();
					window.unminimize();
					window.setFocus();
				}
			});

			// system tray
			const menu = await Menu.new({
				items: [
					{
						id: "show",
						text: "Show Squirrel",
						action: async () => {
							window.setFocus();
							await window.show();
						},
					},
					{
						id: "quit",
						text: "Quit",
						action: async () => {
							exit(0);
						},
					},
				],
			});
			const trayOptions: TrayIconOptions = {
				menu,
			};
			await TrayIcon.new(trayOptions);

			// intercept close event to hide the window instead
			unlisten = await window.onCloseRequested(async (event) => {
				event.preventDefault();
				await window.hide();
			});
		};

		initSetup();

		// cleanup handler
		return () => {
			if (unlisten) {
				unlisten();
			}
		};
	});
</script>

<svelte:window on:keydown={handleNavigation} />
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<main class="container" data-selectable="true">
	<div>
		<p>Squirrel</p>
	</div>
	<div>
		<div class="subhead">
			<p><small>Clipboard history:</small></p>
			<button onclick={clearHistory}>Clear History (permanent!)</button>
		</div>
		{#each cbLog as event, index}
			<Entry
				bind:this={entries[index]}
				content={event.content}
				type={event.event_type}
				timestamp={event.timestamp}
				clickHandler={() => {
					activeIndex = index;
				}} />
		{/each}
	</div>
</main>

<style>
	:root {
		font-family: sans-serif;
		font-size: 16px;
		color: #0f0f0f;
		background-color: #f6f6f6;
		-webkit-text-size-adjust: 100%;
	}

	.container {
		margin: 0;
		display: flex;
		flex-direction: column;
		padding: 0rem 1rem;
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
