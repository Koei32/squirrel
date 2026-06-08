<script lang="ts">
	import { getCurrentWindow } from "@tauri-apps/api/window";

	const window = getCurrentWindow();
	function handleClose() {
		window.close();
	}
</script>

<div class="titlebar">
	<div data-tauri-drag-region class="drag-region"></div>
	<!-- <div class="head">
		<p>Squirrel</p>
	</div> -->
	<!-- https://api.iconify.design/mdi:window-minimize.svg -->
	<!-- <button id="titlebar-minimize" title="minimize">
			<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24">
				<path fill="currentColor" d="M19 13H5v-2h14z" />
			</svg>
		</button> -->

	<!-- https://api.iconify.design/mdi:close.svg -->
	<button id="titlebar-close" title="close" onclick={handleClose}>
		<svg xmlns="http://www.w3.org/2000/svg" width="1rem" height="1rem" viewBox="0 0 24 24">
			<path
				fill="currentColor"
				d="M13.46 12L19 17.54V19h-1.46L12 13.46L6.46 19H5v-1.46L10.54 12L5 6.46V5h1.46L12 10.54L17.54 5H19v1.46z" />
		</svg>
	</button>
</div>

<style>
	/* .head {
		height: 1rem;
		width: 100%;
		margin: 0;
		color: #555;
		text-align: center;
		font-size: 0.75rem;
		align-self: center;
	}

	.head > p {
		margin-top: 0.25rem;
	} */

	.drag-region {
		flex-grow: 1;
	}
	.titlebar {
		height: 1.5rem;
		width: 100%;
		display: flex;
		flex-direction: row;
		justify-content: center;
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
	}

	button {
		all: unset;
		appearance: none;
		margin-right: 0.25rem;
		align-self: center;
		border: none;
		border-radius: 2px;
		display: inline-flex;
		justify-content: center;
		align-items: center;
		width: 0.75rem;
		height: 0.75rem;
		background-color: transparent;
		cursor: pointer;
	}

	svg {
		color: #555;
	}

	button:hover {
		background: #ddd;
	}
</style>
