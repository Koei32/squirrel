<script lang="ts">
	import { listen, type UnlistenFn } from "@tauri-apps/api/event";
	import Entry from "../lib/components/Entry.svelte";
	import type { cbEventNotice, clipboardEvent } from "../lib/types";
	import { readText } from "@tauri-apps/plugin-clipboard-manager";
	import { invoke, Channel } from "@tauri-apps/api/core";
	import { getCurrentWindow } from "@tauri-apps/api/window";
	import { onMount } from "svelte";
	import { TrayIcon, type TrayIconOptions } from "@tauri-apps/api/tray";
	import { Menu } from "@tauri-apps/api/menu";
	import { register } from "@tauri-apps/plugin-global-shortcut";
	import { exit } from "@tauri-apps/plugin-process";

	const window = getCurrentWindow();

	// All events received from the backend
	let cbEvents: Array<clipboardEvent> = $state([]);
	// Entries that are actually displayed due to current search query
	let displayedEntries: Array<Entry> = $state([]);

	// Selection tracking
	let activeIndex: number = $state(0);
	let activeEntry: Entry | undefined = $derived(displayedEntries[activeIndex]);

	// Search stuff
	let searchBar: HTMLInputElement;
	let searchQuery = $state("");
	let filteredCbEvents: Array<clipboardEvent> = $derived(
		cbEvents.filter((event) =>
			event.content.toLowerCase().includes(searchQuery.trim().toLowerCase()),
		),
	);

	// Main listener of backend events
	listen<cbEventNotice>("cb-text-copy", async (event) => {
		const text = readText();
		cbEvents.unshift({
			...event.payload,
			content: await text,
		});
		activeEntry?.focusElement();
	});

	// Channel to stream over history on launch
	const onEvent = new Channel<clipboardEvent>();
	onEvent.onmessage = (event) => {
		cbEvents.push(event);
		console.log(`received ${event.id}`);
	};

	/**
	 * Invokes the clear_history command in the backend and clears all entries
	 */
	async function clearHistory() {
		await invoke("clear_history").catch((r) => {
			console.log(r);
		});
		cbEvents = [];
		displayedEntries = [];
	}

	function pinEntry() {
		// TODO
	}

	// there's probably a better way to do this
	function clamp(num: number, min: number, max: number) {
		return Math.min(Math.max(num, min), max);
	}

	/**
	 * Removes the Entry associated with the passed `id` from the UI.
	 * **Doesn't invoke the delete_entry command in the backend.**
	 * This is supposed to be passed into the `onDelete` attribute of `Entry`.
	 * @param id
	 */
	async function removeEntry(id: number) {
		cbEvents = cbEvents.filter((event) => event.id != id);
		activeIndex = clamp(activeIndex, 0, filteredCbEvents.length - 1);
	}

	// without this, something goes wonky and `displayedEntries` has a null
	// element when the last entry is deleted. a bit confusing.
	$effect(() => {
		displayedEntries.length = filteredCbEvents.length;
	});

	/**
	 * Master keyboard input handler
	 * @param event
	 */
	async function handleKbInput(event: KeyboardEvent) {
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
				activeIndex = clamp(activeIndex + 1, 0, displayedEntries.length - 1);
				activeEntry?.focusElement();
				break;
			}
			case "ArrowUp": {
				event.preventDefault();
				activeIndex = clamp(activeIndex - 1, 0, displayedEntries.length - 1);
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

	onMount(() => {
		let unlisten: UnlistenFn | undefined;

		const initSetup = async () => {
			// Register shortcut to show the window
			await register("CommandOrControl+Shift+V", (event) => {
				if (event.state == "Pressed") {
					window.show();
					window.unminimize();
					window.setFocus();
				}
			});

			// System tray
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

			// Intercept close event to hide the window instead
			unlisten = await window.onCloseRequested(async (event) => {
				event.preventDefault();
				await window.hide();
			});
		};

		initSetup();

		// Load clipboard history
		invoke("load_history", { onEvent });
		activeEntry?.focusElement();

		// Cleanup handler
		return () => {
			if (unlisten) {
				unlisten();
			}
		};
	});

	// TODO: split this giant script block into separate files ideally
</script>

<svelte:window on:keydown={handleKbInput} />
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<main class="container" data-selectable="true">
	<div class="subhead">
		<input
			class="search"
			bind:value={searchQuery}
			bind:this={searchBar}
			placeholder="press / to search" />
	</div>
	<div class="feed">
		{#each filteredCbEvents as event, index (filteredCbEvents[index].id)}
			<Entry
				bind:this={displayedEntries[index]}
				cbEvent={event}
				onClick={() => {
					activeIndex = index;
				}}
				onDelete={removeEntry}
				onPin={pinEntry} />
		{:else}
			<span class="no-history-text">no items</span>
		{/each}
	</div>
</main>

<style>
	:root {
		font-family: system-ui;
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
		margin-bottom: 0.5rem;
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

	/* TODO */
	/* @media (prefers-color-scheme: dark) {
		:root {
			color: #f6f6f6;
			background-color: #2f2f2f;
		}
	} */
</style>
