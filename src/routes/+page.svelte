<script lang="ts">
	import { listen, type UnlistenFn } from "@tauri-apps/api/event";
	import Entry from "../lib/components/Entry.svelte";
	import { cbEventType, type cbEventNotice, type clipboardEvent } from "../lib/types";
	import { invoke, Channel } from "@tauri-apps/api/core";
	import { getCurrentWindow } from "@tauri-apps/api/window";
	import { onMount, tick } from "svelte";

	const window = getCurrentWindow();

	// All events received from the backend
	let cbEvents: Array<clipboardEvent> = $state([]);
	// Entries that are actually displayed due to current search query
	let displayedEntries: Array<Entry> = $state([]);

	let feed: HTMLDivElement | undefined = $state();

	// Selection tracking
	let activeIndex: number = $state(0);
	let activeEntry: Entry | undefined = $derived(displayedEntries[activeIndex]);

	// Search stuff
	let searchBar: HTMLInputElement;
	let searchQuery = $state("");
	let filteredCbEvents: Array<clipboardEvent> = $derived(
		cbEvents
			.filter((event) => {
				if (!searchQuery.trim().toLowerCase()) {
					return event.is_pinned;
				}
				if (event.content.type == cbEventType.Image) return false;

				return event.content.data?.toLowerCase().includes(searchQuery.trim().toLowerCase());
			})
			.concat(
				cbEvents.filter((event) => {
					if (!searchQuery.trim().toLowerCase()) {
						return !event.is_pinned;
					}
				}),
			),
	);

	let noItemText = $state("no items");

	// Channel to stream over content of new entries
	const contentChannel = new Channel<clipboardEvent>();
	contentChannel.onmessage = async (event) => {
		// actually garbage code
		cbEvents.find((entry) => entry.timestamp == event.timestamp)!.id = event.id;
		cbEvents.find((entry) => entry.id == event.id)!.content = event.content;
	};

	// Listener of backend clipboard event notifications
	listen<cbEventNotice>("cb-copy", async (event) => {
		switch (event.payload.event_type) {
			case cbEventType.Text: {
				cbEvents.unshift({
					...event.payload,
					content: { type: event.payload.event_type, data: undefined },
					is_pinned: false,
				});
				break;
			}
			case cbEventType.Image: {
				cbEvents.unshift({
					...event.payload,
					content: { type: event.payload.event_type, data: undefined },
					is_pinned: false,
				});
				break;
			}
		}
	});

	// Channel to stream over history on launch
	const historyChannel = new Channel<clipboardEvent>();
	historyChannel.onmessage = (event) => {
		cbEvents.push(event);
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
		noItemText = "history cleared";
		setTimeout(() => {
			noItemText = "no items";
		}, 2000);
	}

	async function setPinned(id: number) {
		// non-null assertion: setPinned is only called when an Entry of that
		// id exists.
		const was = cbEvents.find((event) => event.id == id)!.is_pinned;
		cbEvents.find((event) => event.id == id)!.is_pinned = !was;
		await tick();
		const position = displayedEntries.findIndex((event) => event.id() == id);
		activeIndex = position;
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
	function removeEntry(id: number) {
		cbEvents = cbEvents.filter((event) => event.id != id);
		activeIndex = clamp(activeIndex, 0, filteredCbEvents.length - 1);
	}

	// without this, something goes wonky and `displayedEntries` has a null
	// element when the last entry is deleted. a bit confusing.
	$effect(() => {
		displayedEntries.length = filteredCbEvents.length;
	});

	$effect(() => {
		cbEvents;
		activeIndex;
		if (document.activeElement !== searchBar) {
			activeEntry?.focusElement();
		}
	});

	/**
	 * Master keyboard input handler
	 * @param event
	 */
	async function handleNavigation(event: KeyboardEvent) {
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
				}
				break;
			}
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
			case "Delete": {
				if ((event.metaKey || event.ctrlKey) && event.shiftKey) {
					clearHistory();
				}
				break;
			}
		}
	}

	onMount(() => {
		let unlisten: UnlistenFn | undefined;

		const initSetup = async () => {
			// Intercept close event to hide the window instead
			unlisten = await window.onCloseRequested(async (event) => {
				event.preventDefault();
				await window.hide();
			});
		};

		initSetup();

		// Load clipboard history
		invoke("load_history", { onEvent: historyChannel });

		invoke("set_event_channel", { channel: contentChannel });

		// Cleanup handler
		return () => {
			if (unlisten) {
				unlisten();
			}
		};
	});

	// TODO: split this giant script block into separate files ideally
</script>

<svelte:window on:keydown={handleNavigation} />
<main class="container" data-selectable="true">
	<div class="subhead">
		<input
			class="search"
			bind:value={searchQuery}
			bind:this={searchBar}
			placeholder="press / to jump to search" />
	</div>
	<div class="feed" bind:this={feed}>
		{#each filteredCbEvents as event, index (filteredCbEvents[index].id)}
			<Entry
				bind:this={displayedEntries[index]}
				cbEvent={event}
				onClick={() => {
					activeIndex = index;
				}}
				onDelete={removeEntry}
				onPin={setPinned} />
		{:else}
			<div class="no-items">
				<svg
					xmlns="http://www.w3.org/2000/svg"
					xml:space="preserve"
					width="175.203mm"
					height="228.639mm"
					version="1.0"
					style="shape-rendering:geometricPrecision; text-rendering:geometricPrecision; image-rendering:optimizeQuality; fill-rule:evenodd; clip-rule:evenodd"
					viewBox="0 0 3852705 5027755"
					class="logo">
					<defs>
						<style type="text/css">
							.str0 {
								stroke-width: 279272;
								stroke-linecap: round;
								stroke-linejoin: round;
								stroke-miterlimit: 22.9256;
							}
							.fil0 {
								fill: none;
							}
						</style>
					</defs>
					<g id="Layer_x0020_1">
						<path
							class="fil0 str0"
							d="M191084 2101478c-32624,-114108 -51448,-188951 -51448,-336450 0,-651832 493874,-1018213 797181,-1574375 20180,-37005 59617,-56384 101248,-49730 41631,6654 73053,37341 80699,78797 86651,469766 282619,1014688 -318390,1732668 -601009,717980 -655885,1002226 -655885,1351219 0,712355 860818,1268322 1434766,1512700 114853,48901 231266,72828 347098,71791 115832,1038 232240,-22885 347098,-71791 573952,-244383 1434766,-800345 1434766,-1512700 0,-348993 -54876,-633240 -655883,-1351219 -601007,-717980 -405043,-1262901 -318392,-1732668 7646,-41455 39067,-72142 80699,-78797 41631,-6654 81068,12726 101248,49730 303307,556163 797181,922543 797181,1574375 0,147500 -18823,222342 -51448,336450" />
						<path
							class="fil0 str0"
							d="M2498898 1859147c-213764,-87329 -373642,-136232 -572545,-136232 -198903,0 -358781,48903 -572545,136232" />
					</g>
				</svg>

				<span class="no-history-text">{noItemText}</span>
			</div>
		{/each}
	</div>
</main>

<style>
	:root {
		--bg-primary: #f6f6f6;
		--bg-accent: #a9a9a9;
		--bg-secondary: #eee;

		--fg-primary: #0f0f0f;
		--fg-accent: #999;

		--entry-focus: #d6d6e1;
		--entry-pinned: #fffcca;
		--entry-pinned-focus: #dddaaf;

		--black: #000; /* ehh */

		font-family: system-ui;
		font-size: 1rem;
		color: var(--fg-primary);
		background-color: var(--bg-primary);
		-webkit-text-size-adjust: 100%;
	}

	.container {
		box-sizing: border-box;
		margin: 0;
		margin-top: 0.5rem;
		height: calc(100vh - 2.75rem);
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
		color: var(--fg-primary);
		box-sizing: border-box;
		background-color: var(--bg-secondary);
		border-radius: 0.2rem;
		border: 1px solid var(--fg-accent);
		height: 1.75rem;
		width: 100%;
		margin: 0;
		padding: 0;
		padding-left: 0.35rem;
	}

	input:focus {
		background-color: var(--bg-primary);
		border: 1px solid var(--fg-primary);
		outline: none;
	}

	.feed {
		box-sizing: border-box;
		scrollbar-gutter: stable;
		scroll-behavior: smooth;
		display: flex;
		flex-direction: column;
		justify-content: start;
		overflow-y: auto;
		padding: 1px;
		padding-right: 3px;
	}

	.no-items {
		display: flex;
		align-items: center;
		flex-direction: column;
	}

	.logo {
		height: 2rem;
		width: 2rem;
		margin-top: 2rem;
		margin-bottom: 0.5rem;
	}

	.str0 {
		stroke-width: 160000;
		stroke: var(--fg-accent);
	}

	.no-history-text {
		text-align: center;
		justify-self: center;
		font-size: 0.75rem;
		color: var(--fg-accent);
	}

	@media (prefers-color-scheme: dark) {
		:root {
			--bg-primary: #0f0f0f;
			--bg-secondary: #202020;
			--bg-accent: #525252;

			--fg-primary: #dedede;
			--fg-accent: #828282;

			--entry-focus: #39393d;
			--entry-pinned: #565439;
			--entry-pinned-focus: #43432c;

			--black: #fff;
		}
	}
</style>
