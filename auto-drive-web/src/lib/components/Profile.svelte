<script lang="ts">
  import { supabase } from "$lib/supabaseClient";

  let loading = $state(false);

  let loginEmail = $state("");
  let loginPassword = $state("");

  let signupEmail = $state("");
  let signupPassword = $state("");

  const handleSignout = async () => {
    try {
      loading = true;

      const { error } = await supabase.auth.signOut({
      });

      if (error) throw error;

    } catch (error) {
      alert(error instanceof Error ? error.message : "Something went wrong.");
    } finally {
      loading = false;
    }
  };
</script>

<div class="row flex-center flex">
  <div class="col-6 form-widget" aria-live="polite">

    <!-- Sign Up -->
    <form
      class="form-widget m-4 p-4"
      onsubmit={(e) => {
        e.preventDefault();
        handleSignout();
      }}
    >
      <h2 class="description">Sign Out</h2>

      <div>
        <button
          type="submit"
          class="button block"
          disabled={loading}
        >
          <span>{loading ? "Loading..." : "Sign Out"}</span>
        </button>
      </div>
    </form>
  </div>
</div>