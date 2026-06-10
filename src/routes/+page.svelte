<script lang="ts">
	import { listen, type UnlistenFn } from "@tauri-apps/api/event";
	import Entry from "../lib/components/Entry.svelte";
	import type { cbEventNotice, clipboardEvent } from "../lib/types";
	import { readText } from "@tauri-apps/plugin-clipboard-manager";
	import { invoke } from "@tauri-apps/api/core";
	import { getCurrentWindow } from "@tauri-apps/api/window";
	import { onMount, tick } from "svelte";
	import { TrayIcon, type TrayIconOptions } from "@tauri-apps/api/tray";
	import { Menu } from "@tauri-apps/api/menu";
	import { register } from "@tauri-apps/plugin-global-shortcut";
	import { exit } from "@tauri-apps/plugin-process";

	const window = getCurrentWindow();

	let cbLog: Array<clipboardEvent> = $state([]);
	let entries: Array<Entry> = $state([]);
	let activeIndex = $state(0);

	listen<cbEventNotice>("cb-text-copy", async (event) => {
		const text = readText();
		cbLog.unshift({
			...event.payload,
			content: await text,
		});
		focusEntry();
	});

	async function clearHistory() {
		await invoke("clear_history").catch((r) => {
			console.log(r);
		});
		cbLog = [];
		entries = [];
	}

	function pinEntry() {
		// todo
	}

	async function deleteEntry(id: number) {
		cbLog = cbLog.filter((event) => event.id != id);
		activeIndex = Math.max(0, (activeIndex -= 1));

		// without this, something goes wonky and `entries` has a null
		// element when the last entry is deleted. a bit confused.
		await tick();
		entries.length = cbLog.length;
	}

	async function handleKbInput(event: KeyboardEvent) {
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
			case "c": {
				entries[activeIndex].handleCopy();
				break;
			}
			case "p": {
				pinEntry();
				break;
			}
			case "Enter": {
				window.hide();
				setTimeout(() => {
					entries[activeIndex].handlePaste();
				}, 50);
				break;
			}
			case "Delete": {
				entries[activeIndex].handleRemove();
				break;
			}
		}
	}

	function focusEntry() {
		if (!entries) return;
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
		focusEntry();

		// cleanup handler
		return () => {
			if (unlisten) {
				unlisten();
			}
		};
	});
</script>

<svelte:window on:keydown={handleKbInput} />
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<main class="container" data-selectable="true">
	<div class="subhead">
		<span><small>Clipboard history:</small></span>
	</div>
	<div class="feed">
		{#each cbLog as event, index (cbLog[index].id)}
			<Entry
				bind:this={entries[index]}
				cbEvent={event}
				onClick={() => {
					activeIndex = index;
				}}
				onDelete={deleteEntry}
				onPin={pinEntry} />
		{:else}
			<span class="no-history-text">no history yet</span>
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
		height: 100vh;
		margin: 0;
		margin-top: 2rem;
		display: flex;
		flex-direction: column;
		padding: 0rem 0.35rem 0rem 0.65rem;
	}

	.subhead {
		flex-direction: row;
		justify-content: space-between;
		align-items: center;
	}

	.feed {
		scrollbar-gutter: stable;
		scroll-behavior: smooth;
		display: flex;
		flex-direction: column;
		justify-content: start;
		height: 80vh; /* ehh */
		overflow-y: auto;
		padding: 1px;
	}

	.no-history-text {
		text-align: center;
		/* align-self: center; */
		justify-self: center;
		font-size: 0.75rem;
		color: #bbb;
	}

	/* @media (prefers-color-scheme: dark) {
		:root {
			color: #f6f6f6;
			background-color: #2f2f2f;
		}
	} */
</style>
