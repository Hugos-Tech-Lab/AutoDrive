<script>
  let darkMode = false;

  function toggleDarkMode() {
    darkMode = !darkMode;
    document.documentElement.classList.toggle("dark", darkMode);
  }
</script>

<button class="dark-toggle" on:click={toggleDarkMode}>
  {darkMode ? "Light" : "Dark"}
</button>

<style>
  .dark-toggle {
    position: fixed;
    top: 1rem;
    right: 1rem;
    z-index: 1000;
    border: 2px solid rgba(0, 0, 0, 0.6);
    border-radius: 5px;
    cursor: pointer;
    font-size: 1.2rem;
    font-family: "Cousin";
    background: none;

    padding: 10px;
    opacity: 0.6;
  }

  :global(.dark) {
    background: #111;
    color: #eee;
  }

  :global(.dark) .dark-toggle {
    /* background: #333; */
    color: white;
  }
</style>
