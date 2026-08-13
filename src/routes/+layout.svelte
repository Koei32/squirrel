<script lang="ts">
	import { onMount } from "svelte";
	import Titlebar from "$lib/components/Titlebar.svelte";
	import "../app.css";
	import { invoke } from "@tauri-apps/api/core";
	import type { Theme } from "$lib/types";
	let { children } = $props();

	let theme: Theme = $state("light");

	/**
	 * Gets the theme from the backend config and applies it.
	 */
	async function setTheme() {
		theme = await invoke("get_theme");
		if (theme == "system") {
			document.documentElement.removeAttribute("data-theme");
		} else {
			document.documentElement.setAttribute("data-theme", theme);
		}
	}

	// Deselect everything when clicked anywhere except data-selectable="true" elements
	function handleDeselectClick(event: MouseEvent): void {
		const target = event.target as HTMLElement;
		if (!target.closest('[data-selectable="true"]')) {
			const selection = window.getSelection();
			if (selection) {
				selection.removeAllRanges();
			}
		}
	}

	onMount(() => {
		setTheme();

		window.addEventListener("click", handleDeselectClick);
		return () => window.removeEventListener("click", handleDeselectClick);
	});
</script>

<div class="master">
	<Titlebar />
	{@render children()}
</div>

<style>
	:root {
		--shadow: inset 0 0 0 1px rgba(0, 0, 0, 0.15);
	}
	.master {
		box-sizing: border-box;
		box-shadow: inset 0 0 0 1px rgba(0, 0, 0, 0.15);
		border: none;
		background-clip: padding-box;
		height: 100vh;
	}

	.master::after {
		content: "";
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		box-shadow: var(--shadow);
		pointer-events: none;
		z-index: 9999;
	}

	@media (prefers-color-scheme: dark) {
		:root {
			--shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.15);
		}
	}
</style>
