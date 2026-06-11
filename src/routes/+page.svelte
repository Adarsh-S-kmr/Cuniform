<script>
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { LogicalSize } from "@tauri-apps/api/dpi";
  import { onMount } from "svelte";

  let tiles = $state([]);
  let errorMsg = $state("");
  let editingId = $state(null);

  async function fetchTiles() {
    try {
      tiles = await invoke("get_tiles");
      errorMsg = "";
      
      const baseWidth = 220;
      const tileWidth = 160;
      const requiredWidth = baseWidth + (tiles.length * tileWidth);
      await getCurrentWindow().setSize(new LogicalSize(requiredWidth, 120));
    } catch (e) {
      errorMsg = `Error fetching tiles: ${e}`;
    }
  }

  async function createTile() {
    try {
      await invoke("create_tile");
      await fetchTiles();
    } catch (e) {
      errorMsg = `Error: ${e}`;
    }
  }

  async function captureWindow(tileId) {
    try {
      errorMsg = "";
      await invoke("capture_active_window_to_tile", { tileId });
      await fetchTiles();
      errorMsg = "Captured!";
      setTimeout(() => errorMsg = "", 2000);
    } catch (e) {
      errorMsg = `Error: ${e}`;
    }
  }

  async function dissolveTile(tileId) {
    try {
      await invoke("dissolve_tile", { tileId });
      await fetchTiles();
    } catch (e) {
      errorMsg = `Error: ${e}`;
    }
  }

  async function toggleVisibility(tileId) {
    try {
      await invoke("toggle_tile_visibility", { tileId });
      // Fetching tiles isn't strictly necessary for visibility since we are manually syncing, but good for consistency
      await fetchTiles(); 
    } catch (e) {
      errorMsg = `Error: ${e}`;
    }
  }

  async function renameTile(tileId, name) {
    try {
      await invoke("update_tile_name", { tileId, name });
      // The local input state handles the visual update, so we don't strictly need fetchTiles, 
      // but we do it to ensure the backend state is synced to the array if needed.
    } catch (e) {
      errorMsg = `Error: ${e}`;
    }
  }

  function startDrag(e) {
    if (e.buttons === 1) {
      getCurrentWindow().startDragging();
    }
  }

  onMount(() => {
    fetchTiles();
    const interval = setInterval(fetchTiles, 1000);
    return () => clearInterval(interval);
  });
</script>

<style>
  :global(body), :global(html) {
    overflow: hidden !important;
    background: transparent !important;
    width: 100vw;
    height: 100vh;
  }
</style>

<main class="w-full h-full rounded-[24px] bg-slate-950/50 backdrop-blur-3xl border border-white/10 flex items-center p-4 shadow-[0_8px_32px_0_rgba(0,0,0,0.6)] m-4 gap-4 overflow-hidden relative" style="width: calc(100vw - 32px); height: calc(100vh - 32px);">
  
  <div onpointerdown={startDrag} role="button" tabindex="0" style="-webkit-app-region: drag; app-region: drag;" class="flex-shrink-0 flex items-center justify-center px-2 cursor-move text-slate-500 hover:text-cyan-400 transition-colors h-full" title="Drag to move Cuneiform">
    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="pointer-events-none"><circle cx="9" cy="12" r="1"></circle><circle cx="9" cy="5" r="1"></circle><circle cx="9" cy="19" r="1"></circle><circle cx="15" cy="12" r="1"></circle><circle cx="15" cy="5" r="1"></circle><circle cx="15" cy="19" r="1"></circle></svg>
  </div>

  <div class="flex-shrink-0 flex flex-col items-center justify-center px-3 border-l border-r border-white/10 h-full gap-2 relative">
    <svg data-tauri-drag-region xmlns="http://www.w3.org/2000/svg" width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="url(#cyan-gradient)" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="pointer-events-none drop-shadow-[0_0_8px_rgba(34,211,238,0.5)]">
      <defs>
        <linearGradient id="cyan-gradient" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stop-color="#22d3ee" />
          <stop offset="100%" stop-color="#818cf8" />
        </linearGradient>
      </defs>
      <path d="M4 4h16v16H4z"></path>
      <path d="M4 12h16"></path>
      <path d="M12 4v16"></path>
    </svg>
    <button onclick={() => invoke("exit_app")} class="text-rose-400/50 hover:text-rose-400 text-[9px] font-bold tracking-widest transition-colors flex items-center justify-center border border-rose-500/20 bg-rose-500/10 rounded px-1.5 py-0.5" title="Close Cuneiform">EXIT</button>
    {#if errorMsg}
      <div class="absolute -bottom-10 left-1/2 -translate-x-1/2 whitespace-nowrap bg-rose-500/10 text-rose-400 px-3 py-1 rounded text-xs border border-rose-500/20 shadow-lg backdrop-blur-md z-50">
        {errorMsg}
      </div>
    {/if}
  </div>

  {#each tiles as tile}
    <div class="flex-shrink-0 bg-white/5 border border-white/10 rounded-xl p-2 flex flex-col w-36 h-full relative group transition-all hover:bg-white/10">
      <div class="flex justify-between items-center mb-1 pb-1">
        <div class="flex-1 min-w-0 mr-1 flex items-center group/edit">
          {#if editingId === tile.id}
            <!-- svelte-ignore a11y_autofocus -->
            <input 
              type="text" 
              value={tile.name || ''} 
              onblur={(e) => { renameTile(tile.id, e.target.value); editingId = null; }}
              onkeydown={(e) => { if(e.key === 'Enter') { renameTile(tile.id, e.target.value); editingId = null; } }}
              class="bg-transparent border-b border-cyan-400 focus:outline-none text-cyan-300 font-mono text-[10px] w-full"
              placeholder="Name..."
              autofocus
            />
          {:else}
            <button 
              onclick={() => toggleVisibility(tile.id)}
              class="text-cyan-300 hover:text-cyan-100 font-mono text-[11px] truncate transition-colors text-left font-bold"
              title="Toggle Visibility"
            >
              {tile.name}
            </button>
            <button 
              onclick={() => editingId = tile.id}
              class="ml-1 opacity-0 group-hover/edit:opacity-100 text-slate-500 hover:text-cyan-400 transition-all"
              title="Edit Name"
            >
              <svg xmlns="http://www.w3.org/2000/svg" width="9" height="9" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 20h9"></path><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"></path></svg>
            </button>
          {/if}
        </div>
        <div class="flex items-center gap-1">
          <button 
            onclick={() => captureWindow(tile.id)}
            class="text-cyan-500 hover:text-cyan-300 transition-colors"
            title="Capture Target Window"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="5" x2="12" y2="19"></line><line x1="5" y1="12" x2="19" y2="12"></line></svg>
          </button>
          <button 
            onclick={() => dissolveTile(tile.id)}
            class="text-slate-500 hover:text-rose-400 transition-colors"
            title="Break Tile (Dissolve)"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>
          </button>
        </div>
      </div>
      
      <div class="flex items-end flex-1 w-full justify-center pb-1">
        <div class="flex items-center text-slate-400 font-mono text-[9px]">
           <span class="bg-cyan-500/20 text-cyan-300 px-1.5 py-0.5 rounded font-bold mr-1">{tile.hwnds.length}</span> windows
        </div>
      </div>
    </div>
  {/each}

  <button 
    onclick={createTile}
    title="Create New Tile"
    aria-label="Create New Tile"
    class="flex-shrink-0 w-8 h-full rounded-xl border border-dashed border-white/20 flex items-center justify-center text-slate-400 hover:text-cyan-400 hover:border-cyan-400/50 hover:bg-cyan-400/5 transition-all group cursor-pointer"
  >
    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="group-hover:scale-110 transition-transform"><line x1="12" y1="5" x2="12" y2="19"></line><line x1="5" y1="12" x2="19" y2="12"></line></svg>
  </button>

</main>
