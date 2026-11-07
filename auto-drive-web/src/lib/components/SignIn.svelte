<script lang="ts">
  import { supabase } from "$lib/supabaseClient";
  import { goto } from "$app/navigation";

  let loading = $state(false);
  let errorMessage = $state("");

  let loginEmail = $state("");
  let loginPassword = $state("");

  let { onSwitchToSignup }: { onSwitchToSignup?: () => void } = $props();

  const handlePasswordLogin = async () => {
    try {
      loading = true;
      errorMessage = ""; // Clear old errors

      const { data, error } = await supabase.auth.signInWithPassword({
        email: loginEmail,
        password: loginPassword
      });

      if (error) throw error;

      goto("/dashboard");
    } catch (error) {
      errorMessage = error instanceof Error ? error.message : "Something went wrong.";
    } finally {
      loading = false;
    }
  };
</script>

<div class="row flex-center flex">
  <div class="col-6 form-widget" aria-live="polite">

    <!-- Password Login -->
    <form
      class="form-widget m-4 p-4"
      onsubmit={(e) => {
        e.preventDefault();
        handlePasswordLogin();
      }}
    >
      <h2 class="description">Sign in via password</h2>

      {#if errorMessage}
        <div class="error-message" style="color: red; margin-bottom: 1rem;">
          {errorMessage}
        </div>
      {/if}

      <div>
        <label for="login-email">Email</label>
        <input
          id="login-email"
          class="inputField"
          type="email"
          placeholder="Your email"
          bind:value={loginEmail}
          required
        />
      </div>

      <div>
        <label for="login-password">Password</label>
        <input
          id="login-password"
          class="inputField"
          type="password"
          placeholder="Your password"
          bind:value={loginPassword}
          required
        />
      </div>

      <div>
        <button
          type="submit"
          class="button block"
          disabled={loading}
        >
          <span>{loading ? "Loading..." : "Sign in"}</span>
        </button>
      </div>

      <div style="margin-top: 1rem; text-align: center;">
        <p>
          Don't have an account? 
          <button 
            type="button" 
            class="link-button" 
            onclick={() => goto("/create-account")}
            style="background: none; border: none; color: blue; text-decoration: underline; cursor: pointer; padding: 0;"
          >
            Create one
          </button>
        </p>
      </div>
    </form>
  </div>
</div>