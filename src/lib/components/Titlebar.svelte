<script lang="ts">
	import { getCurrentWindow } from "@tauri-apps/api/window";

	const window = getCurrentWindow();

	async function handleOptions() {}

	function handleClose() {
		window.close();
	}

	let focused = $state(true);

	window.onFocusChanged(({ payload }) => {
		focused = payload;
	});
</script>

<div class={`titlebar ${focused ? "" : "unfocus"}`} data-tauri-drag-region>
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
			<style>
			</style>
		</defs>
		<path
			class="str0"
			d="M191084 2101478c-32624,-114108 -51448,-188951 -51448,-336450 0,-651832 493874,-1018213 797181,-1574375 20180,-37005 59617,-56384 101248,-49730 41631,6654 73053,37341 80699,78797 86651,469766 282619,1014688 -318390,1732668 -601009,717980 -655885,1002226 -655885,1351219 0,712355 860818,1268322 1434766,1512700 114853,48901 231266,72828 347098,71791 115832,1038 232240,-22885 347098,-71791 573952,-244383 1434766,-800345 1434766,-1512700 0,-348993 -54876,-633240 -655883,-1351219 -601007,-717980 -405043,-1262901 -318392,-1732668 7646,-41455 39067,-72142 80699,-78797 41631,-6654 81068,12726 101248,49730 303307,556163 797181,922543 797181,1574375 0,147500 -18823,222342 -51448,336450" />
		<path
			class="str0"
			d="M2498898 1859147c-213764,-87329 -373642,-136232 -572545,-136232 -198903,0 -358781,48903 -572545,136232" />
	</svg>

	<div class="head">
		<p data-tauri-drag-region>Squirrel</p>
	</div>

	<button id="titlebar-options" title="options" onclick={handleOptions}>
		<svg
			xmlns="http://www.w3.org/2000/svg"
			width="0.8rem"
			height="0.8rem"
			viewBox="0 0 24 24"
			stroke="currentColor"
			stroke-width="2"
			><circle cx="12" cy="12" r="1" /><circle cx="12" cy="5" r="1" /><circle
				cx="12"
				cy="19"
				r="1" /></svg>
	</button>
	<button id="titlebar-close" title="close" onclick={handleClose}>
		<svg xmlns="http://www.w3.org/2000/svg" width="1rem" height="1rem" viewBox="0 0 24 24">
			<path
				fill="currentColor"
				d="M13.46 12L19 17.54V19h-1.46L12 13.46L6.46 19H5v-1.46L10.54 12L5 6.46V5h1.46L12 10.54L17.54 5H19v1.46z" />
		</svg>
	</button>
</div>

<style>
	.titlebar {
		box-sizing: border-box;
		cursor: move;
		background-color: var(--bg-secondary);
		border-bottom: 1px solid var(--fg-accent);
		height: 1.5rem;
		width: 100%;
		display: flex;
		flex-direction: row;
		align-items: center;
		justify-content: center;
	}

	.logo {
		height: 1rem;
		width: 1rem;
		margin-left: 0.2rem;
	}

	.head {
		height: 1rem;
		width: 100%;
		margin: 0;
		color: var(--fg-primary);
		text-align: center;
		font-size: 0.75rem;
		justify-self: center;
	}

	.head > p {
		margin: 0;
	}

	button {
		all: unset;
		appearance: none;
		margin-right: 0.3rem;
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

	.str0 {
		stroke: var(--fg-primary);
		stroke-width: 279272;
		stroke-linecap: round;
		stroke-linejoin: round;
		stroke-miterlimit: 22.9256;
		fill: none;
	}

	button:hover {
		background: var(--bg-accent);
	}

	.unfocus > svg > path {
		stroke: var(--fg-accent);
	}

	.unfocus > button > svg {
		color: var(--fg-accent);
	}

	.unfocus > div > p {
		color: var(--fg-accent);
	}
</style>
