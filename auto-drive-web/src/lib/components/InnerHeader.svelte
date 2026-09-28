<script lang="ts">
  import { goto } from "$app/navigation";
  import UserAccount from "$lib/components/UserAccount.svelte";
  import type { AuthSession } from "@supabase/supabase-js";
  let { children, username }: { children: () => any, username: string | null } = $props();
</script>

<div class="px-4 pt-4 flex items-start gap-8">
  <h1 class="text-4xl mb-4 min-w-0 flex-1">
      {@render children()}
  </h1>

  <div class="flex flex-col items-end gap-2 shrink-0">
    <div class="flex gap-2">
      <UserAccount username={username} />
    </div>
  </div>
</div>