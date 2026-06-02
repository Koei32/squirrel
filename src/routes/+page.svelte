<script lang="ts">
    // import { invoke } from "@tauri-apps/api/core";
    import { listen } from "@tauri-apps/api/event";
    import Entry from "../lib/components/Entry.svelte";

    type clipboardEvent = {
        content: string;
    };

    let cbLog: Array<string> = $state([]);

    listen<clipboardEvent>("cb-text-copy", (event) => {
        cbLog.push(event.payload.content);
    });
</script>

<main class="container">
    <h1>Squirrel</h1>
    <div>
        <h3>clipboard log:</h3>
        {#each cbLog as text}
            <Entry content={text}/>
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
    }

    h1 {
        text-align: center;
    }

    div {
        padding-left: 2rem;
        display: flex;
        flex-direction: column;
    }

    @media (prefers-color-scheme: dark) {
        :root {
            color: #f6f6f6;
            background-color: #2f2f2f;
        }
    }
</style>
