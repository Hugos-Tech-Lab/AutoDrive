<script lang="ts">
  let { label, id }: { label: string; id: string } = $props();

  function scrollToSection(id: string) {
    document.getElementById(id)?.scrollIntoView({
      behavior: "smooth",
      block: "start"
    });
  }
</script>

<button onclick={() => scrollToSection(id)}>{label}</button >
