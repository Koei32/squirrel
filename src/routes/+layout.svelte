<script lang="ts">
	import { onMount } from "svelte";
	import Titlebar from "$lib/components/Titlebar.svelte";
	import "../app.css";
	let { children } = $props();

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
		window.addEventListener("click", handleDeselectClick);
		return () => window.removeEventListener("click", handleDeselectClick);
	});
</script>

<Titlebar />
{@render children()}
