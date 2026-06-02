<script lang="ts">
	import { invoke } from "@tauri-apps/api/core"

    const {content} = $props();

    function copyContent() {
        invoke("copy_content", content);
    }
</script>


<div class="container">
    <span>
        {content}
    </span>
    <button onclick={copyContent}>Copy</button>
</div>


<style>
div{
    height: 2rem;
    width: 80%;
    display: flex;
    justify-content: space-between;
}

</style>