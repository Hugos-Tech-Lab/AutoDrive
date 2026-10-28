<script lang="ts">
  import { supabase } from "$lib/supabaseClient";

  let loading = $state(false);

  let loginEmail = $state("");
  let loginPassword = $state("");

  let signupEmail = $state("");
  let signupPassword = $state("");

  const handlePasswordLogin = async () => {
    try {
      loading = true;

      const { data, error } = await supabase.auth.signInWithPassword({
        email: loginEmail,
        password: loginPassword
      });

      if (error) throw error;

      // Supabase has authenticated the user.
      // Add navigation here if required.
    } catch (error) {
      alert(error instanceof Error ? error.message : "Something went wrong.");
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
    </form>
  </div>
</div>