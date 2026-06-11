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
	let filteredCbLog: Array<clipboardEvent> = $derived(
		cbLog.filter((event) =>
			event.content.toLowerCase().includes(searchQuery.trim().toLowerCase()),
		),
	);
	let displayedEntries: Array<Entry> = $state([]);
	let activeIndex: number | undefined = $state(0);
	let activeEntry: Entry | undefined = $derived(displayedEntries[activeIndex]);

	listen<cbEventNotice>("cb-text-copy", async (event) => {
		const text = readText();
		cbLog.unshift({
			...event.payload,
			content: await text,
		});
		// await tick();
		// displayedEntries.length = cbLog.length;
		activeEntry?.focusElement();
	});

	async function clearHistory() {
		await invoke("clear_history").catch((r) => {
			console.log(r);
		});
		cbLog = [];
		displayedEntries = [];
	}

	function pinEntry() {
		// todo
	}

	function clamp(num: number, min: number, max: number) {
		return Math.min(Math.max(num, min), max);
	}

	async function deleteEntry(id: number) {
		cbLog = cbLog.filter((event) => event.id != id);
		if (activeIndex) {
			activeIndex = clamp(activeIndex, 0, filteredCbLog.length - 1);
		}

		// without this, something goes wonky and `displayedEntries` has a null
		// element when the last entry is deleted. a bit confused.
		// await tick();
		// displayedEntries.length = cbLog.length;
	}

	$effect(() => {
		displayedEntries.length = filteredCbLog.length;
	});

	async function handleKbInput(event: KeyboardEvent) {
		// debug assert
		// if (filteredCbLog.length != displayedEntries.length) {
		// 	console.warn(
		// 		`cblog: ${filteredCbLog.length} != ${displayedEntries.length} :dispentries`,
		// 	);
		// }

		switch (event.key) {
			case "Escape": {
				event.preventDefault();
				window.hide();
				return;
			}
			case "/": {
				if (document.activeElement != searchBar) {
					event.preventDefault();
					searchBar.focus();
					break;
				}
			}
		}

		switch (event.key) {
			case "ArrowDown": {
				event.preventDefault();
				activeIndex = clamp(activeIndex! + 1, 0, displayedEntries.length - 1);
				activeEntry?.focusElement();
				break;
			}
			case "ArrowUp": {
				event.preventDefault();
				activeIndex = clamp(activeIndex! - 1, 0, displayedEntries.length - 1);
				activeEntry?.focusElement();
				break;
			}
			case "c": {
				if (document.activeElement == searchBar) return;
				activeEntry?.handleCopy();
				break;
			}
			case "p": {
				if (document.activeElement == searchBar) return;
				pinEntry();
				break;
			}
			case "Enter": {
				window.hide();
				setTimeout(() => {
					activeEntry?.handlePaste();
				}, 50);
				break;
			}
			case "Delete": {
				if (document.activeElement == searchBar) return;
				activeEntry?.handleRemove();
				break;
			}
			default: {
				searchBar.focus();
			}
		}
	}

	let searchBar: HTMLInputElement;
	let searchQuery = $state("");

	async function searchHandler() {
		console.log(displayedEntries.length);
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
		activeEntry?.focusElement();

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
		<input
			class="search"
			bind:value={searchQuery}
			oninput={searchHandler}
			bind:this={searchBar}
			placeholder="press / to search" />
		<p>Clipboard history:</p>
	</div>
	<div class="feed">
		{#each filteredCbLog as event, index (filteredCbLog[index].id)}
			<Entry
				bind:this={displayedEntries[index]}
				cbEvent={event}
				onClick={() => {
					activeIndex = index;
				}}
				onDelete={deleteEntry}
				onPin={pinEntry} />
		{:else}
			<span class="no-history-text">no items</span>
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
		/* width: 100%; */
		padding-right: 0.3rem;
		flex-direction: column;
		justify-content: space-between;
		align-items: center;
	}

	input {
		border-radius: 0.2rem;
		border: 1px solid #aaa;
		box-sizing: border-box;
		height: 1.5rem;
		width: 100%;
		margin: 0;
		padding: 0;
		padding-left: 0.35rem;
	}

	input:focus {
		border: 1px solid #333;
		outline: none;
	}

	p {
		margin: 0.5rem 0 0 0;
		font-size: 0.8rem;
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
